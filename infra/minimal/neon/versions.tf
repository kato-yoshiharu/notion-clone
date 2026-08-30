terraform {
  required_version = ">= 1.14"

  # stateはinfra/minimal/bootstrapが作るバケットに置く。
  # ローカルから実行するときは AWS_PROFILE=notion-clone を付ける。
  backend "s3" {
    bucket = "kato-yoshiharu-notion-clone-tfstate"
    key    = "neon/terraform.tfstate"
    region = "ap-northeast-1"

    # S3のネイティブロック。DynamoDBのロックテーブルは使わない。
    use_lockfile = true
    encrypt      = true
  }

  required_providers {
    neon = {
      source  = "kislerdm/neon"
      version = "0.14.0"
    }
  }
}
