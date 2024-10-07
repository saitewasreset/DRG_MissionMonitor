# Mission Monitor

Mission Monitor，《深岩银河》游戏数据分析一站式解决方案。

## 子项目

- [Mission Monitor Mod](https://github.com/saitewasreset/DRG_MissionMonitor-mod)
- [Mission Monitor Web UI](https://github.com/saitewasreset/DRG_MissionMonitor-webui)
- [Mission Monitor 后端](https://github.com/saitewasreset/mission-backend-rs)

## 部署

### 服务器端

#### 配置 Docker Secret

配置`docker-compose.yaml`中`secrets`部分的文件。

| 名称           | 含义                                                            |
| -------------- | --------------------------------------------------------------- |
| db_user        | 数据库用户名                                                    |
| db_passwd      | 数据库密码                                                      |
| db_conn_url    | 连接数据库的 URL：`postgres://<db_user>:<db_passwd>@db/monitor` |
| redis_conn_url | 连接 Redis 的 URL：`redis://redis/`                             |
| access_token   | 访问后端管理功能 API 的认证 Token                               |

随后执行

```shell
sudo docker compose up
```

#### 前置 Web 服务器配置

由于`load_mission`上传任务时的请求 Body 较大，若在本项目提供的 Nginx 服务器前仍有前置 Web 服务器，需配置最大允许的请求体长度。

例如，对于 Nginx：

```config
http {
    ...

    client_max_body_size 512M;

    ...
}
```

作为参考，一次性上传 180 次任务的日志，请求 Body 的大小约为 3.5M。

### 客户端

管理工具下载：见[Github Release](https://github.com/saitewasreset/mission-backend-rs/releases)。

客户端需加载 Mod 以获得数据，并将数据初始化。该过程仅需操作一次。随后每次游戏结束后需使用管理工具`load_mission`上传日志文件。

#### 加载 Mod

需使用[mint](https://github.com/trumank/mint)加载子项目中的[Mission Monitor Mod](https://github.com/saitewasreset/DRG_MissionMonitor-mod)，将`.pak`文件拖拽至`mint`界面后，点击`Install mods`即可完成安装。

#### 全局参数配置

全局脚本参数配置选项在`./config.json`文件中。

- access_token：需与服务器端配置的 access_token 一致
- endpoint_url：后端 API 的“根”路径
- mapping_path：存储 mapping 文件的目录（已附带提供）
- watchlist_path：存储要重点分析玩家 ID 的文件
- kpi_config_path：存储 KPI 配置文件的目录（已附带提供）

#### 运行管理工具

以下操作需要按顺序执行：

##### `load_watchlist`

加载要重点分析（在伤害统计、KPI 统计中进行展示）的玩家 ID。

在`watchlist_path`对应的文本文件中写入要重点分析的玩家 ID（每行一个玩家 ID，UTF-8 编码）。

##### `load_kpi`

为了计算玩家 KPI，需要加载 KPI 数据。

##### `load_mapping`

为了将游戏内部的武器、敌人、任务 ID 等与中文名称相匹配，需要加载 mapping。

##### `load_mission`

用于将游戏信息加载到后端，游戏日志文件名应为`MissionMonitor_{timestamp}.txt`（由 Mod 自动生成）。

将所有要加载的游戏日志放到`./raw_log`目录中，已经加载的日志不会被重复加载。

## 更新

每次更新服务器端的容器时，仅需执行下列命令：

```shell
sudo docker compose pull
sudo docker compose up --force-recreate
```
