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

由于上传任务时的请求 Body 较大，若在本项目提供的后端前仍有前置 Web 服务器，需配置最大允许的请求体长度。

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

客户端需加载 Mod 以获得数据，并将数据初始化。该过程仅需操作一次。随后每次游戏结束后需使用管理工具`mission-monitor-tools load-mission`上传日志文件。

#### 加载 Mod

需使用[mint](https://github.com/trumank/mint)加载子项目中的[Mission Monitor Mod](https://github.com/saitewasreset/DRG_MissionMonitor-mod)，将`.pak`文件拖拽至`mint`界面后，点击`Install mods`即可完成安装。

#### 全局参数配置

详见：

`mission-monitor-tools config --help`

#### 运行管理工具

第一次运行时，需向服务端加载配置文件。

首先，进行如下配置：

- 在`config/watchlist.txt`中写入要重点分析的玩家 ID（每行一个玩家 ID，UTF-8 编码）。
- 将所有要加载的游戏日志放到`./raw_log`目录中，已经加载的日志不会被重复加载。游戏日志文件名应为`MissionMonitor_{timestamp}.txt`（由 Mod 自动生成）

之后，进行认证：

`mission-monitor-tools login`

最后，初始化服务端：

`mission-monitor-tools server-init`

## 更新

每次更新服务器端的容器时，仅需执行下列命令：

```shell
sudo docker compose pull
sudo docker compose up --force-recreate
```
