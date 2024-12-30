package main

import (
	"fmt"
	"log"
	"os"
	"time"

	"github.com/PuerkitoBio/goquery"
	readability "github.com/philipjkim/goreadability"
)

func main() {
	url := "https://en.wikipedia.org/wiki/Particle_physics"
	opt := readability.NewOption()

	file, err := os.Open("../wikipedia.html")
	if err != nil {
		log.Fatalf("Failed to open file: %v", err)
	}
	defer file.Close()

	doc, err := goquery.NewDocumentFromReader(file)
	if err != nil {
		log.Fatalf("Failed to create document: %v", err)
	}

	start := time.Now()

	content, err := readability.ExtractFromDocument(doc, url, opt)
	if err != nil {
		log.Fatalf("Failed to extract article: %v", err)
	}

	_ = content

	elapsed := time.Since(start)

	fmt.Printf("readability.go: %s\n", elapsed)
}
