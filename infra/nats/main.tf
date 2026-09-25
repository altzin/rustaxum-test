terraform {
  required_providers {
    jetstream = {
      source  = "nats-io/jetstream"
      version = "~> 0.4.0"
    }
  }
}

provider "jetstream" {
  servers = "nats://127.0.0.1:4222"
}

resource "jetstream_stream" "bookmarks" {
  name      = "BOOKMARKS"
  subjects  = ["bookmark.pending", "bookmark.completed"]
  retention = "workqueue"
  storage   = "file"
  discard   = "old"
  
  max_msgs  = 1000
  max_bytes = 1073741824 # 1GB
}

resource "jetstream_stream" "bookmarks_dlq" {
  name      = "BOOKMARKS_DLQ"
  subjects  = ["bookmark.dlq"]
  retention = "limits"
  storage   = "file"
  discard   = "old"
  
  max_age   = 604800 # 7 days
}
