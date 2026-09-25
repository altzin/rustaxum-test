package main

import (
	"context"
	"os" // Added
	"time"

	"github.com/JohannesKaufmann/html-to-markdown"
	"github.com/chromedp/chromedp"
)

func GenerateMarkdown(targetURL string) ([]byte, error) {
	// 1. Setup custom allocator options for containerized environments
	opts := append(chromedp.DefaultExecAllocatorOptions[:],
		chromedp.Flag("disable-dev-shm-usage", true), // Fixes the 64MB container crash
	)

	// If the CHROME_BIN environment variable is set (via Docker), tell chromedp to use it
	if chromeBin := os.Getenv("CHROME_BIN"); chromeBin != "" {
		opts = append(opts, chromedp.ExecPath(chromeBin))
	}

	// 2. Create the allocator context
	allocCtx, cancel := chromedp.NewExecAllocator(context.Background(), opts...)
	defer cancel()

	// 3. Create the actual browser context with a timeout
	ctx, cancel := chromedp.NewContext(allocCtx)
	defer cancel()

	ctx, cancel = context.WithTimeout(ctx, 45*time.Second)
	defer cancel()

	var htmlBody string

	// Navigate and grab the HTML
	err := chromedp.Run(ctx,
		chromedp.Navigate(targetURL),
		chromedp.WaitVisible(`body`, chromedp.ByQuery),
		chromedp.OuterHTML(`body`, &htmlBody, chromedp.ByQuery),
	)
	if err != nil {
		return nil, err
	}

	converter := md.NewConverter("", true, nil)
	markdown, err := converter.ConvertString(htmlBody)
	if err != nil {
		return nil, err
	}

	return []byte(markdown), nil
}
