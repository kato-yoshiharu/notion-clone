# 開発環境とツールチェーンの修正

## 概要

`pnpm install` が `ERR_PNPM_IGNORED_BUILDS` で失敗したことを起点に調査したところ、
lint / format 系のコマンドが長期間動作していないこと、CI がそれを検知できていないことが判明した。

### 1. pnpm のビルドスクリプト許可設定が読まれていない

pnpm 10 以降、依存パッケージのインストールスクリプトはサプライチェーン対策としてデフォルトで実行されない。
許可リストは `package.json` の以下のフィールドに置かれていた。

```json
  "pnpm": {
    "onlyBuiltDependencies": [
      "esbuild",
      "workerd"
    ]
  }
```

pnpm 11 はこのフィールドを読まなくなったため、ビルドスクリプトが無視され `ERR_PNPM_IGNORED_BUILDS` で失敗していた。

`esbuild` / `workerd` のバイナリはプラットフォーム別パッケージから供給されるため、スクリプトがスキップされても動作自体に影響はない。
問題は pnpm 11 がこれをエラー終了として扱い、`pnpm install` が通らなくなることにある。

### 2. Node のバージョンが `engines` と乖離

`engines.node` は `20.x` だが、ローカルでは mise のグローバル設定（`node = "lts"`）により Node 24 で動いていた。

### 3. `lint:es` と `format` が実行不能

`eslint` / `prettier` が直接依存として宣言されておらず、
`@suimenkathemove/frontend-eslint-config` / `@suimenkathemove/prettier-config` 経由の推移的依存になっていた。

yarn v1 は node_modules をフラットに hoist するため、推移的依存のバイナリも `node_modules/.bin` に入るが、pnpm は直接依存のバイナリしか配置しない。
`01e1d86` での yarn → pnpm 移行後に追加されたこれらのスクリプトは、追加時点から一度も動いていなかった。

### 4. lint の失敗が握り潰されていた

`"lint": "pnpm lint:es; pnpm lint:style"` の区切りが `;` であるため、
`lint:es` の `command not found` は無視され、終了コードは `lint:style` のものになる。
CI の `run: pnpm lint` も同じ経路を通るため、ジョブはグリーンのままだった。

### 5. CI が lockfile を無視していた

CI は `suimenkathemove/pnpm-cache-action@v1.0.0` 経由で pnpm 8 / Node 20 に固定されていたが、
コミット済みの lockfile は `lockfileVersion: '9.0'`（pnpm 9 以降）。
ログには `WARN Ignoring not compatible lockfile` が出力され、毎回ゼロから依存を再解決していた。
ロックされたバージョンは検証されていなかった。
Node 20 固定は 2. の乖離が CI 側でも起きていたことを意味する。

また pnpm 8 はビルドスクリプトを既定で実行するため、`ERR_PNPM_IGNORED_BUILDS` は CI では発生しない。
`1.`の問題はローカル（pnpm 11）でのみ表面化していた。

### 6. lint がビルド成果物を対象にしていた

`out/` を除外する設定が stylelint / eslint のいずれにも無く、ビルド済みの minified なファイルが lint 対象に含まれていた。

### 7. Infrastructure CI が存在しないディレクトリを指していた

`infrastructure-ci.yml` の `working-directory` は `./infrastructure/cdk/notion-clone` だが、
`271f2eb` でディレクトリが `infrastructure/notion-clone` へ移動しており、パスが古いままだった。

## タスク

### 1. pnpm のビルドスクリプト許可設定

- [x] `frontend/pnpm-workspace.yaml` に `allowBuilds` を設定する
- [x] `package.json` の旧 `pnpm.onlyBuiltDependencies` フィールドを削除する

```yaml
allowBuilds:
  esbuild: true
  sharp: false
  workerd: true
```

`sharp` は `false` を明示している。意図的に不許可であることを記録するため。
未ビルドでも `next build` が成功することは確認済み。

### 2. Node / pnpm を最新へ更新

- [x] `frontend/mise.toml` で `node = "24"` / `pnpm = "11"` を固定する
- [x] `engines` を `node: 24.x` / `pnpm: 11.x` へ変更する

### 3. eslint / prettier を直接依存として宣言

- [x] `eslint@^8.57.1` を devDependencies へ追加する
- [x] `prettier@^3.9.6` を devDependencies へ追加する

いずれも共有 config パッケージが要求するレンジ（`eslint: ^8.56.0` / `prettier: ^3.1.1`）に収まる。
単一インスタンスに解決され、プラグイン群と同じ eslint を共有することを `pnpm why` で確認済み。

### 4. lintコマンドの失敗の握り潰しを解消

- [x] `lint` の区切りを `;` から `&&` へ変更する

### 5. lint 対象からビルド成果物を除外

- [x] `.eslintrc.js` の `ignorePatterns` に `out` / `.next` / `storybook-static` / `coverage` を追加する
- [x] `.stylelintignore` を追加し、同じディレクトリを除外する
- [x] `.gitignore` に `/storybook-static/` を追加する

### 6. CI のツールチェーン修正

- [x] `frontend-ci.yml` から `suimenkathemove/pnpm-cache-action` を除去する
- [x] `pnpm/action-setup@v4`（pnpm 11）と `actions/setup-node@v4`（Node 24、`cache: pnpm`）を直接記述する

lockfile 9.0 が読まれるようになり、CI が実際にロックされた依存を検証する。
非推奨の `actions/cache@v3` も併せて除去される。

`node_modules` を削除したうえで、CI の全ジョブ（`tsc` / `lint` / `build` / `build-storybook` / `test`）と
`pnpm install --frozen-lockfile` を Node 24 + pnpm 11 のローカル環境で実行し、通過を確認済み。

### 7. 未フォーマットファイルの解消

`format` が動作していなかった間に蓄積したもの。

- [x] `src/hooks/use-router-query.ts`
- [x] `worker/index.ts`

### 8. Infrastructure CI の working-directory 修正

- [x] `infrastructure-ci.yml` の `working-directory` を `./infrastructure/notion-clone` に修正する
