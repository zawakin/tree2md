package main

import (
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"
)

func TestParseExtList(t *testing.T) {
	tests := []struct {
		input    string
		expected []string
	}{
		{".go,.py", []string{".go", ".py"}},
		{"go,py", []string{".go", ".py"}},
		{".js, .ts, .tsx", []string{".js", ".ts", ".tsx"}},
		{"", []string{}},
		{".GO,.PY", []string{".go", ".py"}}, // case insensitive
	}

	for _, test := range tests {
		result := parseExtList(test.input)
		if test.input == "" {
			if len(result) != 0 {
				t.Errorf("parseExtList(%q) = %v, want empty slice", test.input, result)
			}
		} else if !reflect.DeepEqual(result, test.expected) {
			t.Errorf("parseExtList(%q) = %v, want %v", test.input, result, test.expected)
		}
	}
}

func TestDetectLang(t *testing.T) {
	tests := []struct {
		filename string
		expected string
	}{
		{"test.go", "go"},
		{"script.py", "python"},
		{"app.js", "javascript"},
		{"component.tsx", "tsx"},
		{"style.css", "css"},
		{"query.sql", "sql"},
		{"noext", ""},
		{"", ""},
	}

	for _, test := range tests {
		lang := detectLang(test.filename)
		var result string
		if lang != nil {
			result = lang.Name
		}
		if result != test.expected {
			t.Errorf("detectLang(%q) = %q, want %q", test.filename, result, test.expected)
		}
	}
}

func TestGenerateTruncationMessage(t *testing.T) {
	tests := []struct {
		info     TruncationInfo
		expected string
	}{
		{
			TruncationInfo{TruncateType: "lines", ShownLines: 10, TotalLines: 100},
			"[Content truncated: showing first 10 of 100 lines]",
		},
		{
			TruncationInfo{TruncateType: "bytes", ShownBytes: 1024, TotalBytes: 4096},
			"[Content truncated: showing first 1024 of 4096 bytes]",
		},
		{
			TruncationInfo{
				TruncateType: "both",
				ShownLines:   5,
				TotalLines:   50,
				ShownBytes:   512,
				TotalBytes:   2048,
			},
			"[Content truncated: showing first 5 of 50 lines, 512 of 2048 bytes]",
		},
		{
			TruncationInfo{TruncateType: "unknown"},
			"[Content truncated]",
		},
	}

	for _, test := range tests {
		result := generateTruncationMessage(test.info)
		if result != test.expected {
			t.Errorf("generateTruncationMessage() = %q, want %q", result, test.expected)
		}
	}
}

func TestLoadFileContentWithLimits(t *testing.T) {
	// テスト用の一時ファイルを作成
	tmpDir, err := os.MkdirTemp("", "tree2md_test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	// テストファイルの内容
	testContent := "line1\nline2\nline3\nline4\nline5\n"
	testFile := filepath.Join(tmpDir, "test.txt")
	
	err = os.WriteFile(testFile, []byte(testContent), 0644)
	if err != nil {
		t.Fatal(err)
	}

	tests := []struct {
		name             string
		maxBytes         int
		maxLines         int
		expectedTruncated bool
		expectedLines    int
		expectedType     string
	}{
		{
			name:             "no limits",
			maxBytes:         0,
			maxLines:         0,
			expectedTruncated: false,
			expectedLines:    6, // 5 lines + 1 empty line from split
			expectedType:     "",
		},
		{
			name:             "line limit only",
			maxBytes:         0,
			maxLines:         3,
			expectedTruncated: true,
			expectedLines:    3,
			expectedType:     "lines",
		},
		{
			name:             "byte limit only",
			maxBytes:         10,
			maxLines:         0,
			expectedTruncated: true,
			expectedType:     "bytes",
		},
		{
			name:             "both limits - lines more restrictive",
			maxBytes:         100,
			maxLines:         2,
			expectedTruncated: true,
			expectedLines:    2,
			expectedType:     "lines",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			content, info := loadFileContentWithLimits(testFile, test.maxBytes, test.maxLines)
			
			if info.Truncated != test.expectedTruncated {
				t.Errorf("Expected truncated=%v, got %v", test.expectedTruncated, info.Truncated)
			}
			
			if test.expectedTruncated && info.TruncateType != test.expectedType {
				t.Errorf("Expected truncate type=%q, got %q", test.expectedType, info.TruncateType)
			}
			
			if test.expectedLines > 0 {
				actualLines := len(strings.Split(content, "\n"))
				if actualLines != test.expectedLines {
					t.Errorf("Expected %d lines, got %d", test.expectedLines, actualLines)
				}
			}
			
			// 統計情報の確認
			if info.TotalLines != 6 {
				t.Errorf("Expected total lines=6, got %d", info.TotalLines)
			}
			
			if info.TotalBytes != int64(len(testContent)) {
				t.Errorf("Expected total bytes=%d, got %d", len(testContent), info.TotalBytes)
			}
		})
	}
}

func TestLoadFileContentWithLimitsError(t *testing.T) {
	// 存在しないファイル
	content, info := loadFileContentWithLimits("/nonexistent/file.txt", 0, 0)
	
	if !strings.Contains(content, "Error reading file") {
		t.Errorf("Expected error message, got: %s", content)
	}
	
	if info.Truncated {
		t.Errorf("Expected truncated=false for error case, got true")
	}
}

func TestFilterByExtension(t *testing.T) {
	// テスト用のノード構造を作成
	root := &Node{
		Name:  "root",
		IsDir: true,
		Children: []*Node{
			{Name: "file1.go", IsDir: false},
			{Name: "file2.py", IsDir: false},
			{Name: "file3.txt", IsDir: false},
			{
				Name:  "subdir",
				IsDir: true,
				Children: []*Node{
					{Name: "nested.go", IsDir: false},
					{Name: "nested.js", IsDir: false},
				},
			},
		},
	}

	// .goファイルのみをフィルタ
	filterByExtension(root, []string{".go"})

	// 結果の確認
	if len(root.Children) != 2 { // file1.go と subdir が残る
		t.Errorf("Expected 2 children, got %d", len(root.Children))
	}

	var goFile, subdir *Node
	for _, child := range root.Children {
		if child.Name == "file1.go" {
			goFile = child
		} else if child.Name == "subdir" {
			subdir = child
		}
	}

	if goFile == nil {
		t.Error("file1.go should remain after filtering")
	}

	if subdir == nil {
		t.Error("subdir should remain after filtering")
	} else if len(subdir.Children) != 1 {
		t.Errorf("Expected 1 child in subdir, got %d", len(subdir.Children))
	} else if subdir.Children[0].Name != "nested.go" {
		t.Errorf("Expected nested.go, got %s", subdir.Children[0].Name)
	}
}