# Peregrine

Peregrine 是一个基于 [Pingora](https://github.com/cloudflare/pingora) 构建的轻量级 HTTP 网关，提供反向代理、静态文件服务、健康检查以及常见网关过滤能力。

它适合用作本地开发网关、小型服务入口、前端静态资源服务器，或作为学习 Pingora 网关开发的示例项目。

## 功能特性

- 反向代理：按路径前缀将请求转发到不同 upstream。
- 静态文件：从配置的目录提供 HTML、CSS、JS、JSON、图片等文件。
- 健康检查：可配置健康检查路径，返回 `{"status":"ok"}`。
- CORS：支持配置 `Access-Control-Allow-*` 响应头，并处理预检请求。
- 响应头注入：支持全局自定义响应头。
- 安全响应头：可自动添加 `x-content-type-options`、`x-frame-options`、`referrer-policy`。
- 限流：支持全局限流和路由级限流。
- Basic Auth：支持为单个代理路由启用 Basic Auth。
- 请求日志：支持 text 和 json 两种日志格式，并兼容 `RUST_LOG`。

## 快速开始

### 环境要求

- Rust toolchain
- Cargo

当前项目使用 Rust 2024 edition。

### 构建

```bash
cargo build
```

### 启动

默认读取当前目录下的 `peregrine.yaml`：

```bash
cargo run
```

或指定配置文件：

```bash
cargo run -- --config ./peregrine.yaml
```

启动后会监听配置中的 `listen` 地址，例如：

```text
Peregrine listening on http://0.0.0.0:7777
```

### 校验配置

```bash
cargo run -- --config ./peregrine.yaml validate
```

### 访问示例

```bash
curl http://127.0.0.1:7777/health
curl http://127.0.0.1:7777/
curl http://127.0.0.1:7777/api/users
```

## 配置示例

仓库根目录提供了一个示例配置文件：`peregrine.yaml`。

```yaml
listen: 0.0.0.0:7777
healthPath: /health
staticDir: ./dist
logFormat: text

securityHeaders: true

headers:
  x-powered-by: peregrine
  x-demo: my_hello

cors:
  allowOrigin: "*"
  allowMethods: "GET,POST,OPTIONS"
  allowHeaders: "content-type,authorization"

rateLimit:
  requests: 20
  windowSeconds: 60

proxy:
  - path: /api
    upstream: http://127.0.0.1:9000
    rateLimit:
      requests: 3
      windowSeconds: 600
    basicAuth:
      username: admin
      password: 123456

  - path: /auth
    upstream: http://127.0.0.1:9999
    rateLimit:
      requests: 10
      windowSeconds: 300
    basicAuth:
      username: admin
      password: secret

  - path: /doc
    upstream: http://127.0.0.1:9888
```

## 配置字段

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `listen` | string | `0.0.0.0:8080` | HTTP 服务监听地址 |
| `healthPath` | string | `/__health__` | 健康检查路径，必须以 `/` 开头 |
| `staticDir` | string | `./dist` | 静态文件目录 |
| `logFormat` | string | `text` | 日志格式，可选 `text` 或 `json` |
| `headers` | map | `{}` | 添加到响应中的自定义 header |
| `securityHeaders` | bool | `false` | 是否添加默认安全响应头 |
| `cors` | object | 未启用 | CORS 配置 |
| `rateLimit` | object | 未启用 | 全局限流配置 |
| `proxy` | array | `[]` | 代理路由列表 |

### 代理路由

每个代理路由支持以下字段：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `path` | string | 是 | 代理路径前缀，必须以 `/` 开头 |
| `upstream` | string | 是 | 目标服务地址，支持 `http` 和 `https` |
| `rateLimit` | object | 否 | 路由级限流；未配置时使用全局限流 |
| `basicAuth` | object | 否 | 路由级 Basic Auth |

`rateLimit` 包含：

```yaml
requests: 20
windowSeconds: 60
```

`basicAuth` 包含：

```yaml
username: admin
password: 123456
```

## 请求处理规则

Peregrine 的请求处理顺序如下：

1. CORS 预检请求：命中 `OPTIONS` 预检请求时直接返回 `204`。
2. 健康检查：请求路径等于 `healthPath` 时返回健康检查响应。
3. 反向代理：匹配 `proxy` 中的路由并转发。
4. 静态文件：从 `staticDir` 读取对应文件；访问 `/` 时返回 `index.html`。
5. 404：以上规则都未命中时返回 `404 Not Found`。

代理路由按路径前缀匹配，支持精确匹配和子路径匹配。例如 `path: /api` 会匹配 `/api` 和 `/api/users`。当多个路由同时匹配时，优先使用最长路径前缀。

转发到 upstream 时会剥离匹配到的路由前缀：

| 请求路径 | 路由前缀 | upstream 收到的路径 |
| --- | --- | --- |
| `/api` | `/api` | `/` |
| `/api/users?page=1` | `/api` | `/users?page=1` |

代理请求会追加或更新以下转发相关 header：

- `x-forwarded-for`
- `x-forwarded-proto`
- `x-forwarded-host`
- `via`

## 日志

默认日志级别为 `info`。可以通过 `RUST_LOG` 调整：

```bash
RUST_LOG=debug cargo run
```

使用 JSON 日志：

```yaml
logFormat: json
```

每条请求日志会包含请求方法、路径、状态码、耗时、upstream 和是否被限流等信息。

## 目录结构

```text
src/
  cli/       # 命令行参数、init 和 validate 命令
  config/    # 配置结构、解析和校验
  filter/    # CORS、安全响应头、限流、Basic Auth
  handler/   # 健康检查、静态文件、响应写入、404
  proxy/     # 路由匹配、代理服务、upstream 构建
  server/    # Pingora server 启动逻辑
  util/      # 日志和 MIME 类型工具
```

## 开发命令

```bash
cargo fmt
cargo test
cargo run -- --help
cargo run -- --config ./peregrine.yaml validate
```

## 当前状态

项目仍处于早期开发阶段，适合用于学习、实验和小规模场景。生产环境使用前建议补充更完整的测试、配置示例、部署方式和安全策略。
