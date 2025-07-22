package main

import (
	"os"
	"testing"
)

func TestLoadGitignore(t *testing.T) {
	// テスト用の一時ファイルを作成
	content := `# Comments should be ignored
node_modules/
*.log
!important.log
build/
.DS_Store`

	tmpfile := createTempFile(t, content)
	defer removeTempFile(tmpfile)

	patterns, err := loadGitignore(tmpfile)
	if err != nil {
		t.Fatalf("loadGitignore failed: %v", err)
	}

	expectedPatterns := []struct {
		pattern    string
		isNegation bool
		isDir      bool
	}{
		{"node_modules", false, true},
		{"*.log", false, false},
		{"important.log", true, false},
		{"build", false, true},
		{".DS_Store", false, false},
	}

	if len(patterns) != len(expectedPatterns) {
		t.Errorf("Expected %d patterns, got %d", len(expectedPatterns), len(patterns))
	}

	for i, expected := range expectedPatterns {
		if i >= len(patterns) {
			break
		}
		if patterns[i].pattern != expected.pattern {
			t.Errorf("Pattern %d: expected %q, got %q", i, expected.pattern, patterns[i].pattern)
		}
		if patterns[i].isNegation != expected.isNegation {
			t.Errorf("Pattern %d: expected isNegation=%v, got %v", i, expected.isNegation, patterns[i].isNegation)
		}
		if patterns[i].isDir != expected.isDir {
			t.Errorf("Pattern %d: expected isDir=%v, got %v", i, expected.isDir, patterns[i].isDir)
		}
	}
}

func TestMatchGitignorePattern(t *testing.T) {
	tests := []struct {
		path     string
		pattern  string
		expected bool
	}{
		// 完全一致
		{"test.log", "test.log", true},
		{"src/test.log", "test.log", true},
		
		// ワイルドカードパターン
		{"test.log", "*.log", true},
		{"src/test.log", "*.log", true},
		{"test.txt", "*.log", false},
		
		// ディレクトリパターン
		{"node_modules/package.json", "node_modules", true},
		{"src/node_modules/test.js", "node_modules", true},
		
		// プレフィックスパターン
		{"test_file.txt", "test*", true},
		{"other_file.txt", "test*", false},
	}

	for _, test := range tests {
		result := matchGitignorePattern(test.path, test.pattern)
		if result != test.expected {
			t.Errorf("matchGitignorePattern(%q, %q) = %v, expected %v", 
				test.path, test.pattern, result, test.expected)
		}
	}
}

func TestShouldIgnore(t *testing.T) {
	patterns := []GitignorePattern{
		{pattern: "*.log", isNegation: false, isDir: false},
		{pattern: "important.log", isNegation: true, isDir: false},
		{pattern: "node_modules", isNegation: false, isDir: true},
		{pattern: "dist", isNegation: false, isDir: true},
	}

	tests := []struct {
		path     string
		isDir    bool
		expected bool
	}{
		// ログファイル
		{"test.log", false, true},
		{"important.log", false, false}, // 否定パターンでマッチ
		{"src/debug.log", false, true},
		
		// ディレクトリ
		{"node_modules", true, true},
		{"dist", true, true},
		{"src", true, false},
		
		// 通常のファイル
		{"README.md", false, false},
		{"src/index.js", false, false},
	}

	for _, test := range tests {
		result := shouldIgnore(test.path, test.isDir, patterns)
		if result != test.expected {
			t.Errorf("shouldIgnore(%q, %v) = %v, expected %v", 
				test.path, test.isDir, result, test.expected)
		}
	}
}

// テストヘルパー関数
func createTempFile(t *testing.T, content string) string {
	t.Helper()
	tmpfile, err := os.CreateTemp("", "gitignore_test_")
	if err != nil {
		t.Fatal(err)
	}
	if _, err := tmpfile.Write([]byte(content)); err != nil {
		tmpfile.Close()
		t.Fatal(err)
	}
	if err := tmpfile.Close(); err != nil {
		t.Fatal(err)
	}
	return tmpfile.Name()
}

func removeTempFile(path string) {
	os.Remove(path)
}