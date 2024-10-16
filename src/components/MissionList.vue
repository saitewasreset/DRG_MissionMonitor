<script setup lang="ts">
import { ref, h } from "vue";
import { useMessage, NDataTable, NIcon, NCard, type DataTableColumns } from "naive-ui";

import { WarningAltFilled, CheckmarkFilled, Error } from "@vicons/carbon";

import { useRouter } from "vue-router";

import {
  formatMissionDate,
  formatMissionTime,
  formatMissionDifficulty,
  nFormatter,
} from "@/formatter";

import { translate } from "@/mapping";

import type { Response } from "@/type";

interface MissionInfo {
  missionId: number;
  beginTimestamp: number;
  missionTime: number;
  missionTypeId: string;
  hazardId: number;
  missionResult: number;
  rewardCredit: number;
  missionInvalid: boolean;
  missionInvalidReason: string;
}

function renderValidIcon(valid: boolean) {
  if (valid) {
    return h(NIcon, { color: "green" }, { default: () => h(CheckmarkFilled) });
  } else {
    return h(NIcon, { color: "red" }, { default: () => h(WarningAltFilled) });
  }
}

function renderMissionResultIcon(missionResult: number) {
  if (missionResult === 0) {
    return h(NIcon, { color: "green" }, { default: () => h(CheckmarkFilled) });
  } else if (missionResult === 1) {
    return h(NIcon, { color: "red" }, { default: () => h(WarningAltFilled) });
  } else {
    return h(NIcon, { color: "goldenrod" }, { default: () => h(Error) });
  }
}

function createMissionListColumns(
  missionTypeMapping: Record<string, string> | undefined,
): DataTableColumns<MissionInfo> {
  let missionTypeFilterOptions: { label: string; value: string }[] = [];
  if (missionTypeMapping !== undefined) {
    for (const [sourceName, mappedName] of Object.entries(missionTypeMapping)) {
      missionTypeFilterOptions.push({ label: mappedName, value: sourceName });
    }
  }
  return [
    {
      title: "时间",
      key: "beginTimestamp",
      align: "center",
      defaultSortOrder: "descend",
      sorter(a, b) {
        return a.beginTimestamp - b.beginTimestamp;
      },
      render(row) {
        return formatMissionDate(row.beginTimestamp);
      },
    },
    {
      title: "时长",
      key: "missionTime",
      align: "center",
      sorter(a, b) {
        return a.missionTime - b.missionTime;
      },
      render(row) {
        return formatMissionTime(row.missionTime);
      },
    },
    {
      title: "任务类型",
      key: "missionTypeId",
      align: "center",
      render(row) {
        return translate(row.missionTypeId).value;
      },
      filterMultiple: true,
      filterOptions: missionTypeFilterOptions,
      filter(value, row) {
        return row.missionTypeId === value;
      },
    },
    {
      title: "难度",
      key: "hazardId",
      align: "center",
      render(row) {
        return formatMissionDifficulty(row.hazardId);
      },
      filterOptions: [
        { label: "一级风险", value: 1 },
        { label: "二级风险", value: 2 },
        { label: "三级风险", value: 3 },
        { label: "四级风险", value: 4 },
        { label: "五级风险", value: 5 },
        { label: "深潜 阶段一 3", value: 100 },
        { label: "深潜 阶段二 3.5", value: 101 },
        { label: "深潜 阶段三 3.5", value: 102 },
        { label: "精英深潜 阶段一 4.5", value: 103 },
        { label: "精英深潜 阶段二 5", value: 104 },
        { label: "精英深潜 阶段三 5.5", value: 105 },
      ],
      filterMultiple: true,
      filter(value, row) {
        return row.hazardId === value;
      },
    },
    {
      title: "任务状态",
      key: "missionResult",
      align: "center",
      render(row) {
        return renderMissionResultIcon(row.missionResult);
      },
      filterOptions: [
        { label: "已完成", value: 0 },
        { label: "失败", value: 1 },
        { label: "放弃", value: 2 },
      ],
      filterMultiple: true,
      filter(value, row) {
        return row.missionResult === value;
      },
    },
    {
      title: "奖励代币",
      key: "rewardCredit",
      align: "center",
      sorter(a, b) {
        return a.rewardCredit - b.rewardCredit;
      },
      render(row) {
        return nFormatter(row.rewardCredit, 2);
      },
    },
    {
      title: "有效性",
      key: "missionInvalid",
      align: "center",
      render(row) {
        return renderValidIcon(!row.missionInvalid);
      },
      filterOptions: [
        { label: "有效", value: 0 },
        { label: "无效", value: 1 },
      ],
      filterMultiple: true,
      filter(value, row) {
        if (value === 0) {
          return !row.missionInvalid;
        } else {
          return row.missionInvalid;
        }
      },
    },
  ];
}

const router = useRouter();
const message = useMessage();
const missionList = ref<MissionInfo[]>([]);
const missionTypeMapping = ref<Record<string, string>>();

const lastSelectedMissionId = ref<number | null>(null);

const rowProps = (row: MissionInfo) => {
  return {
    style: "cursor: pointer;",
    onClick: () => {
      router.push({ name: "missionDetails", params: { id: row.missionId.toString() } });
    },
  };
};

fetch(`./api/mission/mission_list`)
  .then((res) => res.json())
  .then(
    (res: Response<{ missionInfo: MissionInfo[]; missionTypeMapping: Record<string, string> }>) => {
      if (res.code !== 200) {
        message.error(`API Error while loading mission list: ${res.code} ${res.message}`);
      } else {
        missionList.value = res.data.missionInfo;
        missionTypeMapping.value = res.data.missionTypeMapping;
      }
    },
  )
  .catch((e) => {
    message.error(`HTTP Error while loading mission list: ${e}`);
  });
</script>

<template>
  <n-card title="任务列表">
    <div class="table-container">
      <n-data-table
        :columns="createMissionListColumns(missionTypeMapping)"
        :data="missionList"
        :pagination="{ pageSize: 10 }"
        :row-props="rowProps"
        :row-class-name="
          (rowData) => (rowData.missionId === lastSelectedMissionId ? 'selected-row' : '')
        "
        :row-key="(row: MissionInfo) => row.missionId"
        style="width: fit-content"
      ></n-data-table>
    </div>
  </n-card>
</template>

<style scoped>
.table-container {
  display: flex;
  justify-content: space-around;
}
</style>
