package main

import (
	"flag"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
	"strings"
)

// Node はファイル/ディレクトリのツリー構造を表します。
type Node struct {
	Name     string
	Path     string
	IsDir    bool
	Children []*Node

	// ツリー出力や内容出力で使う
	Content string // コードブロック用に読み込んだファイル内容
}

// グローバルオプション
var (
	flagContents   bool   // -c, --contents
	flagTruncate   int    // --truncate
	flagIncludeExt string // --include-ext
)

func main() {
	// オプション定義
	flag.BoolVar(&flagContents, "C", false, "Include file contents (code blocks)")
	flag.BoolVar(&flagContents, "contents", false, "Include file contents (code blocks)")
	flag.IntVar(&flagTruncate, "truncate", 0, "Truncate file content to the first N bytes")
	flag.StringVar(&flagIncludeExt, "include-ext", "", "Comma-separated list of extensions to include (e.g. .go,.py)")

	// パース
	flag.Parse()

	// ディレクトリ指定（引数なければカレント）
	dir := "."
	if len(flag.Args()) > 0 {
		dir = flag.Args()[0]
	}

	// ツリー構築
	rootNode, err := buildTree(dir)
	if err != nil {
		log.Fatal(err)
	}

	// --include-ext が指定されていれば、対象外ファイルを除去
	if flagIncludeExt != "" {
		exts := parseExtList(flagIncludeExt)
		filterByExtension(rootNode, exts)
	}

	// Markdown 出力: ファイルツリー
	fmt.Println("## File Structure")
	printTree(rootNode, "")

	// -c ( --contents ) が指定されていれば、ツリー上のファイルに対してコードブロックを追加表示
	if flagContents {
		printCodeBlocks(rootNode)
	}
}

// buildTree は指定したパス以下を再帰的に探索し、Node の階層構造を作る
func buildTree(path string) (*Node, error) {
	info, err := os.Stat(path)
	if err != nil {
		return nil, err
	}
	node := &Node{
		Name:  info.Name(),
		Path:  path,
		IsDir: info.IsDir(),
	}

	if info.IsDir() {
		entries, err := ioutil.ReadDir(path)
		if err != nil {
			return node, nil // 読み込み不可なら子なしで返す
		}
		for _, e := range entries {
			childPath := filepath.Join(path, e.Name())
			childNode, err := buildTree(childPath)
			if err == nil {
				node.Children = append(node.Children, childNode)
			}
		}
	}
	return node, nil
}

// parseExtList は "--include-ext=.go,.py" のような文字列をパースして拡張子スライスを返す
func parseExtList(extString string) []string {
	parts := strings.Split(extString, ",")
	var exts []string
	for _, p := range parts {
		e := strings.TrimSpace(strings.ToLower(p))
		// 先頭に '.' が無ければ付ける
		if !strings.HasPrefix(e, ".") {
			e = "." + e
		}
		exts = append(exts, e)
	}
	return exts
}

// filterByExtension はノードを再帰的にたどり、指定された拡張子以外のファイルを除去
// ディレクトリは残すが、中身が空ならそのまま空ツリーになる
func filterByExtension(node *Node, exts []string) {
	if !node.IsDir {
		// ファイルなら、拡張子が含まれているかどうかチェック
		ext := strings.ToLower(filepath.Ext(node.Name))
		for _, e := range exts {
			if ext == e {
				// 該当拡張子 => 残す
				return
			}
		}
		// いずれの拡張子にもマッチしない => ノードを無効化
		node.Name = ""
		return
	}
	// ディレクトリの場合、子要素を再帰的にフィルタ
	for i := 0; i < len(node.Children); i++ {
		child := node.Children[i]
		filterByExtension(child, exts)
		// ファイル名が空になった子は削除
		if child.Name == "" {
			// スライスからの削除
			node.Children = append(node.Children[:i], node.Children[i+1:]...)
			i--
		}
	}
}

// printTree は Markdown形式でツリーを表示する
func printTree(node *Node, indent string) {
	// ルートだけ先に出力（- .）
	if indent == "" {
		fmt.Printf("- %s/\n", node.Name)
	}
	// node がディレクトリなら、その子を表示
	for i, child := range node.Children {
		// 一応、最後の子かどうかでインデントを切り替える例
		isLast := (i == len(node.Children)-1)
		bullet := "  - "
		if isLast {
			bullet = "  - "
		}
		// ディレクトリ名に "/" を付ける
		dirName := child.Name
		if child.IsDir {
			dirName += "/"
		}
		fmt.Printf("%s%s%s\n", indent, bullet, dirName)

		if child.IsDir {
			// インデントを増やして再帰
			printTree(child, indent+"    ")
		}
	}
}

// printCodeBlocks はファイルノードを深さ優先でたどり、コードブロックを出力する
func printCodeBlocks(node *Node) {
	if !node.IsDir {
		// ファイルの場合にのみコードブロックを出力
		// ファイル内容を取得（truncate 有効なら先頭 N バイトだけ読み込む）
		content := loadFileContent(node.Path, flagTruncate)

		// 言語推定
		lang := detectLang(node.Name)

		// ### 見出し
		fmt.Printf("\n### %s\n", node.Path)
		fmt.Printf("```%s\n", lang.Name)
		// おまけでファイル名をコメントに入れる
		// fmt.Printf("// %s\n", node.Name)
		fmt.Printf("%s\n", lang.ToComment(node.Path))
		// コードブロック内のコメントアウト処理
		fmt.Print(content)
		fmt.Println("```")
	}
	for _, child := range node.Children {
		printCodeBlocks(child)
	}
}

// loadFileContent はファイルを開き、truncate があれば指定バイトまで読み込んで返す
func loadFileContent(path string, truncate int) string {
	f, err := os.Open(path)
	if err != nil {
		return fmt.Sprintf("// Error reading file: %v\n", err)
	}
	defer f.Close()

	// truncate == 0 の場合は制限なしで全部読む
	if truncate <= 0 {
		data, _ := ioutil.ReadAll(f)
		return string(data)
	}

	// トランケートする場合
	buf := make([]byte, truncate)
	n, err := f.Read(buf)
	// n バイトだけ読み込み、残りを捨てる
	return string(buf[:n])
}

// detectLang は拡張子に応じてコードブロックの言語名を推定する
func detectLang(filename string) *Lang {
	ext := strings.ToLower(filepath.Ext(filename))
	for _, lang := range langs {
		if lang.Ext == ext {
			// return lang.Name
			return &lang
		}
	}
	return nil
}

type Lang struct {
	Ext       string
	Name      string
	ToComment func(string) string
}

var langs = []Lang{
	{".go", "go", func(s string) string { return "// " + s }},
	{".py", "python", func(s string) string { return "# " + s }},
	{".sh", "shell", func(s string) string { return "# " + s }},
	{".js", "javascript", func(s string) string { return "// " + s }},
	{".ts", "typescript", func(s string) string { return "// " + s }},
	{".tsx", "tsx", func(s string) string { return "// " + s }},
	{".html", "html", func(s string) string { return "<!-- " + s + " -->" }},
	{".css", "css", func(s string) string { return "/* " + s + " */" }},
	{".scss", "scss", func(s string) string { return "/* " + s + " */" }},
	{".sass", "sass", func(s string) string { return "/* " + s + " */" }},
	{".sql", "sql", func(s string) string { return "-- " + s }},
}
