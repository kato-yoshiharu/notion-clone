data "cloudflare_account_api_token_permission_groups_list" "all" {
  account_id = var.account_id
}

locals {
  # 権限グループ名はスコープ内でしか一意でないため、先にスコープで索引を分ける。
  permission_group_ids = {
    for scope in ["com.cloudflare.api.account", "com.cloudflare.api.account.zone"] :
    scope => {
      for group in data.cloudflare_account_api_token_permission_groups_list.all.result :
      group.name => group.id if contains(group.scopes, scope)
    }
  }

  account_permission_group_names = [
    "Workers Scripts Write",
    "Workers KV Storage Write",
  ]

  zone_permission_group_names = [
    "Zone Write",
    "DNS Write",
    "Workers Routes Write",
    "Zone WAF Write",
  ]
}

resource "cloudflare_account_token" "main" {
  account_id = var.account_id
  name       = "notion-clone-ci"

  # スコープごとにポリシーを分ける。
  # どちらのセレクタも同一のアカウントリソースをキーに持つため、
  # アカウントスコープとゾーンスコープの権限グループは1つのポリシーに同居できない。
  policies = [
    {
      effect = "allow"
      permission_groups = [
        for name in local.account_permission_group_names :
        { id = local.permission_group_ids["com.cloudflare.api.account"][name] }
      ]
      resources = jsonencode({
        "com.cloudflare.api.account.${var.account_id}" = "*"
      })
    },
    {
      effect = "allow"
      permission_groups = [
        for name in local.zone_permission_group_names :
        { id = local.permission_group_ids["com.cloudflare.api.account.zone"][name] }
      ]
      # アカウント所有トークンはフラットな "com.cloudflare.api.account.zone.*" を受け付けないため、ゾーンのセレクタをアカウントリソースの下にネストする。
      resources = jsonencode({
        "com.cloudflare.api.account.${var.account_id}" = {
          "com.cloudflare.api.account.zone.*" = "*"
        }
      })
    },
  ]
}
