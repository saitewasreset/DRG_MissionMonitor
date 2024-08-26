# Mission Monitor

Mission Monitor，《深岩银河》游戏数据分析一站式解决方案。

## 子项目

- [Mission Monitor Mod](https://github.com/saitewasreset/DRG_MissionMonitor-mod)
- [Mission Monitor Web UI](https://github.com/saitewasreset/DRG_MissionMonitor-webui)
- [Mission Monitor 后端](https://github.com/saitewasreset/DRG_MissionMonitor-backend)

## 部署

### 环境变量配置

在`docker-compose.yaml`中，需根据实际情况配置相应环境变量。

#### `db`：

| 名称                | 含义 |
| ------------------- | ---- |
| MYSQL_DATABASE      | -    |
| MYSQL_USER          | -    |
| MYSQL_PASSWORD      | -    |
| MYSQL_ROOT_PASSWORD | -    |

#### `backend`：

| 名称         | 含义               |
| ------------ | ------------------ |
| DB_HOST      | 要连接的数据库主机 |
| DB_DATABASE  | 要连接的数据库名称 |
| DB_USER      | 数据库用户名       |
| DB_PASSWORD  | 数据库密码         |
| ADMIN_PREFIX | 管理功能 URL 前缀  |

### 一键部署

`$ sudo docker compose up`

### 数据上传

使用本项目提供的 load_mission、load_mapping 脚本（见 Release）。
