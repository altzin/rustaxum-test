package main

//
// import (
// 	"crypto/sha256"
// 	"fmt"
// 	"log"
// )
//
// // GenerateStorageKey creates a deterministic SHA-256 hash of the URL
// func GenerateStorageKey(url string) string {
// 	hash := sha256.Sum256([]byte(url))
// 	return fmt.Sprintf("%x.md", hash)
// }
//
// type StorageBackend struct {
// 	// Add AWS S3 / Minio client here
// }
//
// func NewStorageBackend() *StorageBackend {
// 	return &StorageBackend{}
// }
//
// func (s *StorageBackend) Exists(key string) bool {
// 	// TODO: Implement actual S3 HeadObject check
// 	return false
// }
//
// func (s *StorageBackend) Upload(key string, data []byte) error {
// 	// TODO: Implement actual S3 PutObject upload
// 	log.Printf("Mock upload: saved %d bytes to %s", len(data), key)
// 	return nil
// }
