package main

import (
	"fmt"
	"log"
	"os"
	"time"

	"github.com/PuerkitoBio/goquery"
	readability "github.com/philipjkim/goreadability"
	"golang.org/x/net/html"
)

func main() {
	url := "https://en.wikipedia.org/wiki/Particle_physics"
	opt := readability.NewOption()
	opt.LookupOpenGraphTags = false

	file, err := os.Open("../wikipedia.html")
	if err != nil {
		log.Fatalf("Can't open file: %v", err)
	}
	defer file.Close()

	parse_opts := html.ParseOptionEnableScripting(false)
	node, err := html.ParseWithOptions(file, parse_opts)

	if err != nil {
		fmt.Println("Can't parse HTML:", err)
	}

	document := goquery.NewDocumentFromNode(node)
	if err != nil {
		log.Fatalf("Can't create document: %v", err)
	}

	start := time.Now()

	content, err := readability.ExtractFromDocument(document, url, opt)

	elapsed := time.Since(start)

	if err != nil {
		log.Fatalf("Can't extract article: %v", err)
	}

	fmt.Printf("content: %s\n", content.Description)

	fmt.Printf("readability.go: %s\n", elapsed)
}
