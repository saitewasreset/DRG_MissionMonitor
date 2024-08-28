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

### 数据初始化与日志文件加载（`./script`）

#### 脚本运行环境

数据初始化与日志文件加载脚本使用 Python 3 编写，同时依赖 requests 库。

```shell
$ pip3 install requests
```

此外，对于 Windows 系统，也可从[项目 Release](https://github.com/saitewasreset/DRG_MissionMonitor/releases)中下载打包好的版本。

假设部署后后端 API 的根目录为`url = http://127.0.0.1:8080/api`

### `load_hero`

加载游戏中的角色列表。

脚本参数配置：

在`./script/load_hero/config.json`中完成如下配置：

- admin_endpoint：`url/ADMIN_PREFIX`

使用`./script/load_hero/load_hero.py`脚本即可完成加载。

### `load_friends`

加载要重点分析（在伤害统计、KPI 统计中进行展示）的玩家 ID。

脚本参数配置：

在`./script/load_friends/config.json`中完成如下配置：

- admin_endpoint：`url/ADMIN_PREFIX`

在`./script/load_friends/friends.txt`中完成如下配置（该文件需使用 UTF-8 编码，已附带示例）：

- 每行一个游戏 ID

使用`./script/load_friends/load_friends.py`脚本即可完成加载。

### `load_mapping`

为了将游戏内部的武器、敌人、任务 ID 等与中文名称相匹配，需要加载 mapping。

脚本参数配置：

在`./script/load_mapping/config.json`中完成如下配置：

- admin_endpoint：`url/ADMIN_PREFIX`
- mapping_path：存储 mapping 文件的目录（以附带提供，要获得更新，见[主项目地址](https://github.com/saitewasreset/DRG_MissionMonitor)）

使用`./script/load_mapping/load_mapping.py`脚本即可完成加载。

### `load_kpi`

为了计算玩家 KPI，需要加载 KPI 数据。

脚本参数配置：

在`./script/load_kpi/config.json`中完成如下配置：

- admin_endpoint：`url/ADMIN_PREFIX`

使用`./script/load_kpi/load_kpi.py`脚本即可完成加载。

### `load_mission`

用于将游戏信息加载到后端，游戏日志文件名应为`MissionMonitor_{timestamp}.txt`（由 Mod 自动生成）。

脚本参数配置：

在`./script/load_mission/config.json`中完成如下配置：

- admin_endpoint：`url/ADMIN_PREFIX`
- log_path：存储游戏日志文件的目录（`./script/load_mission/log`目录中已有测试数据）

使用`./script/load_mission/load_mission.py`脚本即可完成加载。
