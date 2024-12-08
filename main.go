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
	flag.StringVar(&pattern, "pattern", "", "pattern to filter files")
	flag.StringVar(&langStr, "lang", "en", "language for UI (en or ja)")
	flag.StringVar(&mode, "mode", "full", "output mode (tree or full)")
	flag.Parse()

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

	rootNode, codeFilesPaths, err := buildTree(absRoot, showAll, pattern, mode)
	if err != nil {
		log.Fatal(err)
	}

	extConfig := defaultExtConfig()

	var codeFiles []CodeFile
	for _, file := range codeFilesPaths {
		ext := filepath.Ext(file)
		cfg, ok := extConfig[ext]
		if !ok {
			continue
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
		// treeモード: ファイル構成（ファイルも表示）, コードスニペットはなし
		fmt.Println(getMessage("heading_file_structure", langStr))
		fmt.Print(printMarkdownTree(rootNode, 0))
	case "full":
		// fullモード: ファイル構成 + コードスニペット表示
		fmt.Println(getMessage("heading_file_structure", langStr))
		fmt.Print(printMarkdownTree(rootNode, 0))
		for _, cf := range codeFiles {
			fmt.Printf("\n### %s\n```%s\n%s %s\n%s\n```\n",
				cf.FilePath,
				cf.Lang,
				cf.CommentPrefix,
				cf.FilePath,
				cf.Content,
			)
		}
	default:
		// 不明なモードはfull扱い
		fmt.Println(getMessage("heading_file_structure", langStr))
		fmt.Print(printMarkdownTree(rootNode, 0))
		for _, cf := range codeFiles {
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

func buildTree(root string, showAll bool, pattern string, mode string) (*Node, []string, error) {
	rootNode := &Node{Name: ".", IsDir: true}
	var codeFiles []string

	nodeMap := map[string]*Node{
		root: rootNode,
	}

	shouldDisplayFile := func(name string) bool {
		if pattern != "" {
			matched, _ := filepath.Match(pattern, name)
			return matched
		}
		if showAll {
			return true
		}
		// デフォルトはfullモードでファイルを表示するように変更
		// （これで mode 未指定またはfull時にはファイル表示）
		if mode == "full" {
			return true
		}
		// treeモードの場合もファイル表示する仕様に変更
		if mode == "tree" {
			// 隠しファイルは--allがない限り表示しない
			if strings.HasPrefix(name, ".") && !showAll {
				return false
			}
			return true
		}
		return false
	}

	shouldDisplayDir := func(name string) bool {
		if pattern != "" {
			if strings.HasPrefix(name, ".") && !showAll {
				return false
			}
			return true
		}
		if showAll {
			return true
		}
		if strings.HasPrefix(name, ".") && name != "." {
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
			}
			ext := filepath.Ext(d.Name())
			if ext == ".py" || ext == ".go" {
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
