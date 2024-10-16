# Mission Monitor 后端（Rust）

Mission Monitor，《深岩银河》游戏数据分析一站式解决方案。

[主项目地址](https://github.com/saitewasreset/DRG_MissionMonitor)

## API 文档

[参见](./api.md)

## 部署

### 一键部署

建议使用[主项目](https://github.com/saitewasreset/DRG_MissionMonitor)，利用 Docker compose 进行一键部署。

### 单独构建

本项目已经包含 Dockerfile，可使用 Docker 进行构建。

依赖：

- 数据库：postgresql
- 缓存：redis

已内置 migrations，可自动创建需要的数据库表结构。

标有`(_FILE)`的环境变量支持设置从文件中读取配置，可与 Docker secret 结合。

例如：`A(_FILE)`表示先尝试读取环境变量`A_FILE`的值，并将该变量指向的文件的内容作为`A`的实际配置值；若环境变量`A_FILE`未设置，则尝试读取环境变量`A`，并将其值作为`A`的实际配置值。

环境变量：
| 名称 | 含义 |
| ---- | ---- |
| DATABASE_URL(\_FILE) |符合 PostgreSQL 连接格式的 URL |
| REDIS_URL(\_FILE) | 符合 Redis 连接格式的 URL |
|ACCESS_TOKEN(\_FILE)| 管理功能的 Access Token|
|INSTANCE_DIR(\_FILE)| 保存后端工作数据的目录|

## 管理工具

管理工具集：`load_kpi、load_mapping、load_mission、load_watchlist`

运行：`cargo run --release --bin <tool_name>`

配置参见`config/config.json`

默认读取配置文件路径为`PWD/config.json`，可通过`CONFIG_PATH`环境变量设置。

### 初始化

对于需要详细分析游戏数据的玩家，将其游戏用户名加入`watchlist.txt`中；
对于需要当作**输出型**侦察的玩家，将其游戏用户名加入`mapping/scout_special.txt`中。

**按序**执行：

- `load_watchlist`
- `load_kpi`
- `load_mapping`
- `load_mission`
