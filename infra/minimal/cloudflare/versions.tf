terraform {
  required_version = ">= 1.14"

  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 5.13"
    }
  }
}
