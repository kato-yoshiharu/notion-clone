data "cloudflare_account_api_token_permission_groups_list" "all" {
  account_id = var.account_id
}

locals {
  permission_group_names = [
    "Workers Scripts Write",
    # main_token.tf自身が引く権限グループ一覧の参照に必要。
    "Account API Tokens Read",
  ]

  # 権限グループ名はスコープ内でしか一意でないため、アカウントスコープに絞って索引を作る。
  permission_group_ids = {
    for group in data.cloudflare_account_api_token_permission_groups_list.all.result :
    group.name => group.id if contains(group.scopes, "com.cloudflare.api.account")
  }
}

resource "cloudflare_account_token" "main" {
  account_id = var.account_id
  name       = "notion-clone-ci"

  policies = [
    {
      effect = "allow"
      permission_groups = [
        for name in local.permission_group_names :
        { id = local.permission_group_ids[name] }
      ]
      resources = jsonencode({
        "com.cloudflare.api.account.${var.account_id}" = "*"
      })
    },
  ]
}
