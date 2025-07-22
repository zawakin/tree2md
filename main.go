package main

import (
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
	"strings"
)

const (
	version = "v0.1.5"
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

// TruncationInfo はファイル内容のtruncate情報を保持します。
type TruncationInfo struct {
	Truncated    bool
	TotalLines   int
	TotalBytes   int64
	ShownLines   int
	ShownBytes   int64
	TruncateType string // "bytes", "lines", "both"
}

// グローバルオプション
var (
	flagContents       bool   // -c, --contents
	flagTruncate       int    // --truncate
	flagMaxLines       int    // --max-lines
	flagIncludeExt     string // --include-ext
	flagAll            bool   // -a, --all
	flagRespectGitignore bool   // --respect-gitignore

	// Version
	flagVersion bool
)

func main() {
	// オプション定義
	flag.BoolVar(&flagContents, "c", false, "Include file contents (code blocks)")
	flag.BoolVar(&flagContents, "contents", false, "Include file contents (code blocks)")
	flag.IntVar(&flagTruncate, "t", 0, "Truncate file content to the first N bytes")
	flag.IntVar(&flagTruncate, "truncate", 0, "Truncate file content to the first N bytes")
	flag.IntVar(&flagMaxLines, "max-lines", 0, "Limit file content to the first N lines")
	flag.StringVar(&flagIncludeExt, "e", "", "Comma-separated list of extensions to include (e.g. .go,.py)")
	flag.StringVar(&flagIncludeExt, "include-ext", "", "Comma-separated list of extensions to include (e.g. .go,.py)")
	flag.BoolVar(&flagAll, "a", false, "Include hidden files and directories")
	flag.BoolVar(&flagAll, "all", false, "Include hidden files and directories")
	flag.BoolVar(&flagRespectGitignore, "respect-gitignore", false, "Respect .gitignore files")

	// Version
	flag.BoolVar(&flagVersion, "v", false, "Print version information")
	flag.BoolVar(&flagVersion, "version", false, "Print version information")

	// パース
	flag.Parse()

	if flagVersion {
		fmt.Printf("tree2md %s\n", version)
		os.Exit(0)
	}

	// ディレクトリ指定（引数なければカレント）
	dir := "."
	if len(flag.Args()) > 0 {
		dir = flag.Args()[0]
	}

	// gitignoreパターンの読み込み
	var gitignorePatterns []GitignorePattern
	if flagRespectGitignore {
		gitignorePath := filepath.Join(dir, ".gitignore")
		patterns, err := loadGitignore(gitignorePath)
		if err == nil {
			gitignorePatterns = patterns
		}
		// .gitignoreが存在しない場合はエラーを無視
	}

	// ツリー構築
	rootNode, err := buildTreeWithGitignore(dir, gitignorePatterns)
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
	return buildTreeWithGitignore(path, nil)
}

// buildTreeWithGitignore はgitignoreパターンを考慮してツリーを構築します
func buildTreeWithGitignore(path string, gitignorePatterns []GitignorePattern) (*Node, error) {
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
		entries, err := os.ReadDir(path)
		if err != nil {
			return node, nil // 読み込み不可なら子なしで返す
		}
		for _, e := range entries {
			// -a / --all が指定されていない場合は「ドットで始まる」ものをスキップ
			if !flagAll && strings.HasPrefix(e.Name(), ".") {
				continue
			}

			childPath := filepath.Join(path, e.Name())

			// gitignoreチェック
			if len(gitignorePatterns) > 0 {
				// ルートからの相対パスを計算
				relPath, _ := filepath.Rel(".", childPath)
				if shouldIgnore(relPath, e.IsDir(), gitignorePatterns) {
					continue
				}
			}

			childNode, err := buildTreeWithGitignore(childPath, gitignorePatterns)
			if err == nil {
				node.Children = append(node.Children, childNode)
			}
		}
	}
	return node, nil
}

// parseExtList は "--include-ext=.go,.py" のような文字列をパースして拡張子スライスを返す
func parseExtList(extString string) []string {
	if extString == "" {
		return []string{}
	}
	parts := strings.Split(extString, ",")
	var exts []string
	for _, p := range parts {
		e := strings.TrimSpace(strings.ToLower(p))
		if e == "" {
			continue
		}
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
		// ファイル内容を取得（truncateや行数制限有効なら制限する）
		content, truncated := loadFileContentWithLimits(node.Path, flagTruncate, flagMaxLines)

		// 言語推定
		lang := detectLang(node.Name)

		langName := ""
		if lang != nil {
			langName = lang.Name
		}

		// ### 見出し
		fmt.Printf("\n### %s\n", node.Path)
		fmt.Printf("```%s\n", langName)
		if lang != nil {
			fmt.Printf("%s\n", lang.ToComment(node.Path))
		}
		// コードブロック内のコメントアウト処理
		fmt.Print(content)
		// ファイル末尾に改行がない場合は明示的に追加
		if !strings.HasSuffix(content, "\n") {
			fmt.Print("\n")
		}
		if truncated.Truncated {
			message := generateTruncationMessage(truncated)
			if lang != nil {
				fmt.Printf("%s\n", lang.ToComment(message))
			} else {
				fmt.Printf("// %s\n", message)
			}
		}
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
		data, _ := io.ReadAll(f)
		return string(data)
	}

	// トランケートする場合
	buf := make([]byte, truncate)
	n, err := f.Read(buf)
	// n バイトだけ読み込み、残りを捨てる
	return string(buf[:n])
}

// loadFileContentWithLimits はファイルを開き、バイト数制限と行数制限の両方を考慮して読み込む
func loadFileContentWithLimits(path string, maxBytes, maxLines int) (string, TruncationInfo) {
	f, err := os.Open(path)
	if err != nil {
		return fmt.Sprintf("// Error reading file: %v\n", err), TruncationInfo{}
	}
	defer f.Close()

	// 全体を一度読み込んで統計情報を取得
	data, err := io.ReadAll(f)
	if err != nil {
		return fmt.Sprintf("// Error reading file: %v\n", err), TruncationInfo{}
	}

	content := string(data)
	lines := strings.Split(content, "\n")
	// 末尾の空文字列要素を除去（最後に改行がある場合の対応）
	if len(lines) > 0 && lines[len(lines)-1] == "" {
		lines = lines[:len(lines)-1]
	}
	totalLines := len(lines)
	totalBytes := int64(len(data))

	info := TruncationInfo{
		TotalLines: totalLines,
		TotalBytes: totalBytes,
	}

	// 制限なしの場合
	if maxBytes <= 0 && maxLines <= 0 {
		info.ShownLines = totalLines
		info.ShownBytes = totalBytes
		return content, info
	}

	var truncatedContent string
	var truncatedByBytes, truncatedByLines bool

	// 行数制限を適用
	if maxLines > 0 && totalLines > maxLines {
		lines = lines[:maxLines]
		truncatedByLines = true
	}

	// 一度行数制限を適用した内容を文字列に戻す
	truncatedContent = strings.Join(lines, "\n")

	// バイト数制限を適用
	if maxBytes > 0 && int64(len(truncatedContent)) > int64(maxBytes) {
		truncatedContent = truncatedContent[:maxBytes]
		truncatedByBytes = true
	}

	info.Truncated = truncatedByBytes || truncatedByLines
	info.ShownBytes = int64(len(truncatedContent))
	// ShownLines の計算でも同様の処理を適用
	shownLines := strings.Split(truncatedContent, "\n")
	if len(shownLines) > 0 && shownLines[len(shownLines)-1] == "" {
		shownLines = shownLines[:len(shownLines)-1]
	}
	info.ShownLines = len(shownLines)

	// truncateタイプを設定
	if truncatedByBytes && truncatedByLines {
		info.TruncateType = "both"
	} else if truncatedByBytes {
		info.TruncateType = "bytes"
	} else if truncatedByLines {
		info.TruncateType = "lines"
	}

	return truncatedContent, info
}

// generateTruncationMessage はtruncation情報からメッセージを生成する
func generateTruncationMessage(info TruncationInfo) string {
	switch info.TruncateType {
	case "lines":
		return fmt.Sprintf("[Content truncated: showing first %d of %d lines]", info.ShownLines, info.TotalLines)
	case "bytes":
		return fmt.Sprintf("[Content truncated: showing first %d of %d bytes]", info.ShownBytes, info.TotalBytes)
	case "both":
		return fmt.Sprintf("[Content truncated: showing first %d of %d lines, %d of %d bytes]", 
			info.ShownLines, info.TotalLines, info.ShownBytes, info.TotalBytes)
	default:
		return "[Content truncated]"
	}
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

// GitignorePattern は.gitignoreのパターンを表します
type GitignorePattern struct {
	pattern    string
	isNegation bool
	isDir      bool
}

// loadGitignore は.gitignoreファイルを読み込み、パターンのスライスを返します
func loadGitignore(path string) ([]GitignorePattern, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	var patterns []GitignorePattern
	lines := strings.Split(string(data), "\n")
	for _, line := range lines {
		// 空行とコメント行をスキップ
		line = strings.TrimSpace(line)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}

		pattern := GitignorePattern{pattern: line}

		// 否定パターン
		if strings.HasPrefix(line, "!") {
			pattern.isNegation = true
			pattern.pattern = line[1:]
		}

		// ディレクトリ指定
		if strings.HasSuffix(pattern.pattern, "/") {
			pattern.isDir = true
			pattern.pattern = strings.TrimSuffix(pattern.pattern, "/")
		}

		patterns = append(patterns, pattern)
	}

	return patterns, nil
}

// shouldIgnore は指定されたパスがgitignoreパターンにマッチするかチェックします
func shouldIgnore(path string, isDir bool, patterns []GitignorePattern) bool {
	// 相対パスに変換（./で始まる場合は除去）
	path = strings.TrimPrefix(path, "./")

	ignored := false
	for _, pattern := range patterns {
		if pattern.isDir && !isDir {
			// ディレクトリパターンだがファイルの場合はスキップ
			continue
		}

		if matchGitignorePattern(path, pattern.pattern) {
			if pattern.isNegation {
				ignored = false
			} else {
				ignored = true
			}
		}
	}

	return ignored
}

// matchGitignorePattern は簡易的なgitignoreパターンマッチングを行います
func matchGitignorePattern(path, pattern string) bool {
	// 完全一致
	if path == pattern {
		return true
	}

	// パスのベース名での一致
	if filepath.Base(path) == pattern {
		return true
	}

	// ワイルドカードを含むパターンの簡易的な処理
	if strings.Contains(pattern, "*") {
		// 簡易的な実装: * を任意の文字列として扱う
		// 例: *.log -> .logで終わるかチェック
		if strings.HasPrefix(pattern, "*") {
			suffix := strings.TrimPrefix(pattern, "*")
			if strings.HasSuffix(path, suffix) || strings.HasSuffix(filepath.Base(path), suffix) {
				return true
			}
		}
		// 例: test* -> testで始まるかチェック
		if strings.HasSuffix(pattern, "*") {
			prefix := strings.TrimSuffix(pattern, "*")
			if strings.HasPrefix(path, prefix) || strings.HasPrefix(filepath.Base(path), prefix) {
				return true
			}
		}
	}

	// パスの任意の部分でマッチ（例: node_modules/）
	pathParts := strings.Split(path, "/")
	for _, part := range pathParts {
		if part == pattern {
			return true
		}
	}

	return false
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
