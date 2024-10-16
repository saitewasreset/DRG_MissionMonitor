<script setup lang="ts">
import { ref, h, watch } from "vue";
import {nFormatter, generateKPICharacterClass, getKPICharacterName } from "@/formatter";
import { NAlert, NTag, useMessage, NDataTable, type DataTableColumns } from "naive-ui";

import type { Response } from "@/type";

interface KPIInfo {
  version: string;
}

interface KPIComponent {
  name: string;
  sourceValue: number;
  weightedValue: number;
  missionTotalWeightedValue: number;
  rawIndex: number;
  correctedIndex: number;
  transformedIndex: number;
  weight: number;
}

interface MissionKPIInfo {
  playerName: string;
  kpiCharacterType: string;
  weightedKill: number;
  weightedDamage: number;
  priorityDamage: number;
  reviveNum: number;
  deathNum: number;
  friendlyFire: number;
  nitra: number;
  supplyCount: number;
  weightedResource: number;
  component: KPIComponent[];
  missionKPI: number;
}

interface KPIDataTableRow {
  playerName: string;
  kpiCharacterType: string;
  weightedKill: number;
  weightedDamage: number;
  priorityDamage: number;
  reviveNum: number;
  deathNum: number;
  friendlyFire: number;
  nitra: number;
  supplyCount: number;
  weightedResource: number;
  component: KPIComponent[];
  missionKPI: number;
}

interface KPIDataTableExpandRow {
  name: string;
  sourceValue: number;
  weightedValue: number;
  missionTotalWeightedValue: number;
  rawIndex: number;
  correctedIndex: number;
  transformedIndex: number;
  weight: number;
}

function createKPITableExpandElement(row: KPIDataTableRow) {
  const columns: DataTableColumns<KPIDataTableExpandRow> = [
    {
      title: "项目",
      key: "name",
      align: "center",
    },
    {
      title: "玩家原始值",
      key: "sourceValue",
      align: "center",
      render(row) {
        return nFormatter(row.sourceValue, 2);
      },
    },
    {
      title: "玩家加权值",
      key: "weightedValue",
      align: "center",
      render(row) {
        return nFormatter(row.weightedValue, 2);
      },
    },
    {
      title: "任务合计加权值",
      key: "missionTotalWeightedValue",
      align: "center",
      render(row) {
        return nFormatter(row.missionTotalWeightedValue, 2);
      },
    },
    {
      title: "原始指标",
      key: "rawIndex",
      align: "center",
      render(row) {
        return row.rawIndex.toFixed(3);
      },
    },
    {
      title: "修正后指标",
      key: "correctedIndex",
      align: "center",
      render(row) {
        return row.correctedIndex.toFixed(3);
      },
    },
    {
      title: "赋分后指标",
      key: "transformedIndex",
      align: "center",
      render(row) {
        return row.transformedIndex.toFixed(3);
      },
    },
    {
      title: "权重",
      key: "weight",
      align: "center",
      render(row) {
        return row.weight.toFixed(3);
      },
    },
    {
      title: "加权值",
      key: "weightValue",
      align: "center",
      render(row) {
        return (row.transformedIndex * row.weight).toFixed(3);
      },
    },
  ];

  const data: KPIDataTableExpandRow[] = row.component.map((component) => {
    return {
      name: component.name,
      sourceValue: component.sourceValue,
      weightedValue: component.weightedValue,
      missionTotalWeightedValue: component.missionTotalWeightedValue,
      rawIndex: component.rawIndex,
      correctedIndex: component.correctedIndex,
      transformedIndex: component.transformedIndex,
      weight: component.weight,
    };
  });

  return h(NDataTable, { columns, data, style: "width: fit-content" });
}

function createKPITableColumns(): DataTableColumns<KPIDataTableRow> {
  return [
    {
      type: "expand",
      expandable: () => true,
      renderExpand: (row) => createKPITableExpandElement(row),
    },
    {
      title: "玩家",
      key: "playerName",
      align: "center",
    },
    {
      title: "角色",
      key: "kpiCharacterType",
      align: "center",
      render(row) {
        return h(
          "span",
          { class: generateKPICharacterClass(row.kpiCharacterType) },
          getKPICharacterName(row.kpiCharacterType),
        );
      },
    },
    {
      title: "加权伤害",
      key: "weightedDamage",
      align: "center",
      render(row) {
        return nFormatter(row.weightedDamage, 2);
      },
    },
    {
      title: "加权击杀数",
      key: "weightedKill",
      align: "center",
    },
    {
      title: "高威胁目标",
      key: "priorityDamage",
      align: "center",
      render(row) {
        return nFormatter(row.priorityDamage, 2);
      },
    },
    {
      title: "救人次数",
      key: "reviveNum",
      align: "center",
    },
    {
      title: "倒地次数",
      key: "deathNum",
      align: "center",
    },
    {
      title: "友伤",
      key: "friendlyFire",
      align: "center",
      render(row) {
        return nFormatter(row.friendlyFire, 2);
      },
    },
    {
      title: "硝石采集量",
      key: "nitra",
      align: "center",
      render(row) {
        return row.nitra.toFixed(0);
      },
    },
    {
      title: "补给份数",
      key: "supplyCount",
      align: "center",
    },
    {
      title: "加权资源采集量",
      key: "resourceTotal",
      align: "center",
      render(row) {
        return row.weightedResource.toFixed(0);
      },
    },
    {
      title: "任务KPI",
      key: "missionKPI",
      align: "center",
      render(row) {
        return (row.missionKPI * 100).toFixed(2);
      },
    },
  ];
}

function generateKPITableData(missionData: MissionKPIInfo[]): KPIDataTableRow[] {
  let result = [];
  result = missionData.map((missionData) => {
    return {
      playerName: missionData.playerName,
      kpiCharacterType: missionData.kpiCharacterType,
      weightedKill: missionData.weightedKill,
      weightedDamage: missionData.weightedDamage,
      priorityDamage: missionData.priorityDamage,
      reviveNum: missionData.reviveNum,
      deathNum: missionData.deathNum,
      friendlyFire: missionData.friendlyFire,
      nitra: missionData.nitra,
      supplyCount: missionData.supplyCount,
      weightedResource: missionData.weightedResource,
      component: missionData.component,
      missionKPI: missionData.missionKPI,
    };
  });

  return result;
}

const props = defineProps<{
  missionId?: number;
}>();

const message = useMessage();

const kpiInfo = ref<KPIInfo>({
  version: "0.0.0",
});

const missionKpiInfo = ref<MissionKPIInfo[]>([]);

const kpiTableData = ref<KPIDataTableRow[]>([]);

watch(
  () => props.missionId,
  () => {
    fetch("./api/kpi/version")
      .then((response) => response.json())
      .then((data: Response<KPIInfo>) => {
        if (data.code !== 200) {
          message.error(`API Error while loading KPI Info: ${data.code} ${data.message}`);
        } else {
          kpiInfo.value = data.data;
        }
      })
      .catch((e) => {
        message.error(`HTTP Error while loading KPI Info: ${e}`);
      });

    fetch(`./api/mission/${props.missionId === undefined ? 1 : props.missionId}/kpi`)
      .then((response) => response.json())
      .then((data: Response<MissionKPIInfo[]>) => {
        if (data.code !== 200) {
          message.error(`API Error while loading Mission KPI Info: ${data.code} ${data.message}`);
        } else {
          missionKpiInfo.value = data.data;
          kpiTableData.value = generateKPITableData(data.data);
        }
      })
      .catch((e) => {
        message.error(`HTTP Error while loading Mission KPI Info: ${e}`);
      });
  },
  { immediate: true },
);
</script>
<template>
  <n-tag type="info">当前KPI版本：{{ kpiInfo.version }}</n-tag>
  <div class="warning-div">
    <n-alert title="注意" type="warning" style="width: fit-content"
      >KPI计算仍处于测试阶段！</n-alert
    >
  </div>
  <div class="table-container">
    <n-data-table
      :columns="createKPITableColumns()"
      :data="kpiTableData"
      :pagination="false"
      :row-key="(row: KPIDataTableRow) => row.playerName + row.kpiCharacterType"
      style="width: fit-content"
    ></n-data-table>
  </div>
</template>

<style scoped>
.table-container {
  display: flex;
  justify-content: space-around;
}
.warning-div {
  display: flex;
}
</style>
