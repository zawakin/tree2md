package main

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// 境界値テスト追加
func TestLoadFileContentWithLimitsBoundaryValues(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "tree2md_test_boundary")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	tests := []struct {
		name           string
		content        string
		maxBytes       int
		maxLines       int
		expectedError  bool
	}{
		{
			name:     "max lines = 1",
			content:  "line1\nline2\nline3\n",
			maxLines: 1,
		},
		{
			name:     "max bytes = 1",
			content:  "hello world",
			maxBytes: 1,
		},
		{
			name:     "negative max lines",
			content:  "line1\nline2\n",
			maxLines: -1,
		},
		{
			name:     "negative max bytes",
			content:  "hello",
			maxBytes: -1,
		},
		{
			name:     "zero values",
			content:  "test content",
			maxBytes: 0,
			maxLines: 0,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			testFile := filepath.Join(tmpDir, test.name+".txt")
			err := os.WriteFile(testFile, []byte(test.content), 0644)
			if err != nil {
				t.Fatal(err)
			}

			content, info := loadFileContentWithLimits(testFile, test.maxBytes, test.maxLines)
			
			// 負の値やゼロの場合は制限なしとして動作すべき
			if test.maxBytes <= 0 && test.maxLines <= 0 {
				if info.Truncated {
					t.Errorf("Should not be truncated when limits are <= 0")
				}
				if content != test.content {
					t.Errorf("Content should be unchanged when no limits")
				}
			}
			
			// 有効な制限の場合の基本チェック
			if test.maxBytes > 0 || test.maxLines > 0 {
				if test.maxBytes > 0 && len(content) > test.maxBytes {
					t.Errorf("Content exceeds byte limit: %d > %d", len(content), test.maxBytes)
				}
				
				if test.maxLines > 0 {
					lines := strings.Split(content, "\n")
					if len(lines) > 0 && lines[len(lines)-1] == "" {
						lines = lines[:len(lines)-1]
					}
					if len(lines) > test.maxLines {
						t.Errorf("Content exceeds line limit: %d > %d", len(lines), test.maxLines)
					}
				}
			}
		})
	}
}