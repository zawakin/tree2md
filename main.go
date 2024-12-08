package main

import (
	"flag"
	"fmt"
	"io/fs"
	"log"
	"os"
	"path/filepath"
	"strings"
)

type Node struct {
	Name     string
	IsDir    bool
	Children []*Node
}

type CodeFile struct {
	FilePath      string
	Lang          string
	CommentPrefix string
	Content       string
}

func main() {
	var (
		showAll bool
		pattern string
		langStr string
		mode    string
	)

	flag.BoolVar(&showAll, "all", false, "show all files including hidden")
	flag.StringVar(&pattern, "pattern", "", "pattern to filter files (comma-separated)")
	flag.StringVar(&langStr, "lang", "en", "language for UI (en or ja)")
	flag.StringVar(&mode, "mode", "tree", "output mode (tree or full)")
	flag.Parse()

	// patternが未指定なら"*"にする
	if pattern == "" {
		pattern = "*"
	}

	// patternを","で分割することで複数パターン対応
	patterns := strings.Split(pattern, ",")

	args := flag.Args()
	if len(args) < 1 {
		fmt.Println("Usage: context-cli [--all] [--pattern=...] [--lang=en|ja] [--mode=tree|full] <directory>")
		return
	}

	rootDir := args[0]

	absRoot, err := filepath.Abs(rootDir)
	if err != nil {
		log.Fatal(err)
	}

	rootNode, codeFilesPaths, err := buildTree(absRoot, showAll, patterns, mode)
	if err != nil {
		log.Fatal(err)
	}

	extConfig := defaultExtConfig()

	var codeFiles []CodeFile
	for _, file := range codeFilesPaths {
		ext := filepath.Ext(file)
		cfg, ok := extConfig[ext]
		if !ok {
			// 未対応拡張子は言語指定なし
			cfg = struct {
				Lang          string
				CommentPrefix string
			}{
				Lang:          "",
				CommentPrefix: "",
			}
		}
		content, err := os.ReadFile(file)
		if err != nil {
			log.Printf("failed to read %s: %v", file, err)
			continue
		}
		contentStr := strings.TrimRight(string(content), "\n")
		relPath, err := filepath.Rel(absRoot, file)
		if err != nil {
			relPath = file
		}
		relPath = strings.TrimPrefix(relPath, "./")

		codeFiles = append(codeFiles, CodeFile{
			FilePath:      relPath,
			Lang:          cfg.Lang,
			CommentPrefix: cfg.CommentPrefix,
			Content:       contentStr,
		})
	}

	switch mode {
	case "tree":
		fmt.Println(getMessage("heading_file_structure", langStr))
		fmt.Print(printMarkdownTree(rootNode, 0))
	case "full":
		fmt.Println(getMessage("heading_file_structure", langStr))
		fmt.Print(printMarkdownTree(rootNode, 0))
		for _, cf := range codeFiles {
			if cf.Lang == "" {
				// 言語未指定の場合は ``` のみで囲む
				fmt.Printf("\n### %s\n```\n%s %s\n%s\n```\n",
					cf.FilePath,
					cf.CommentPrefix,
					cf.FilePath,
					cf.Content,
				)
			} else {
				// 対応言語の場合
				fmt.Printf("\n### %s\n```%s\n%s %s\n%s\n```\n",
					cf.FilePath,
					cf.Lang,
					cf.CommentPrefix,
					cf.FilePath,
					cf.Content,
				)
			}
		}
	default:
		// 不明なモードはfull扱い
		fmt.Println(getMessage("heading_file_structure", langStr))
		fmt.Print(printMarkdownTree(rootNode, 0))
		for _, cf := range codeFiles {
			if cf.Lang == "" {
				fmt.Printf("\n### %s\n```\n%s %s\n%s\n```\n",
					cf.FilePath,
					cf.CommentPrefix,
					cf.FilePath,
					cf.Content,
				)
			} else {
				fmt.Printf("\n### %s\n```%s\n%s %s\n%s\n```\n",
					cf.FilePath,
					cf.Lang,
					cf.CommentPrefix,
					cf.FilePath,
					cf.Content,
				)
			}
		}
	}
}

func buildTree(root string, showAll bool, patterns []string, mode string) (*Node, []string, error) {
	rootNode := &Node{Name: ".", IsDir: true}
	var codeFiles []string

	nodeMap := map[string]*Node{
		root: rootNode,
	}

	shouldDisplayFile := func(name string) bool {
		// 複数パターンのいずれかにマッチで表示
		for _, p := range patterns {
			p = strings.TrimSpace(p)
			matched, _ := filepath.Match(p, name)
			if matched {
				if !showAll && strings.HasPrefix(name, ".") {
					return false
				}
				return true
			}
		}
		return false
	}

	shouldDisplayDir := func(name string) bool {
		if !showAll && strings.HasPrefix(name, ".") && name != "." {
			return false
		}
		return true
	}

	err := filepath.WalkDir(root, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if path == root {
			return nil
		}

		relPath, _ := filepath.Rel(root, path)
		parentDir := filepath.Dir(relPath)
		parentNode, exists := nodeMap[filepath.Join(root, parentDir)]
		if !exists {
			return nil
		}

		if d.IsDir() {
			if shouldDisplayDir(d.Name()) {
				node := &Node{Name: d.Name(), IsDir: true}
				parentNode.Children = append(parentNode.Children, node)
				nodeMap[filepath.Join(root, relPath)] = node
			} else {
				return fs.SkipDir
			}
		} else {
			if shouldDisplayFile(d.Name()) {
				node := &Node{Name: d.Name(), IsDir: false}
				parentNode.Children = append(parentNode.Children, node)
				// patternにマッチしたファイルを全てcodeFiles対象にする
				codeFiles = append(codeFiles, path)
			}
		}
		return nil
	})

	return rootNode, codeFiles, err
}

func printMarkdownTree(node *Node, indent int) string {
	var sb strings.Builder
	prefix := strings.Repeat("  ", indent) + "- " + node.Name + "\n"
	sb.WriteString(prefix)

	for _, child := range node.Children {
		sb.WriteString(printMarkdownTree(child, indent+1))
	}
	return sb.String()
}

func defaultExtConfig() map[string]struct {
	Lang          string
	CommentPrefix string
} {
	return map[string]struct {
		Lang          string
		CommentPrefix string
	}{
		".py": {Lang: "python", CommentPrefix: "#"},
		".go": {Lang: "go", CommentPrefix: "//"},
	}
}

var messages = map[string]map[string]string{
	"heading_file_structure": {
		"en": "## File Structure",
		"ja": "## ファイル構成",
	},
}

func getMessage(key, lang string) string {
	langs, ok := messages[key]
	if !ok {
		return key
	}
	msg, ok := langs[lang]
	if !ok {
		if fallback, fok := langs["en"]; fok {
			return fallback
		}
		return key
	}
	return msg
}
