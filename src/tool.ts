interface GroupInfo {
  labelList: string[];
  dataList: number[];
}

export const characterFilterOptions = [
  { label: "钻机", value: "DRILLER" },
  { label: "枪手", value: "GUNNER" },
  { label: "工程", value: "ENGINEER" },
  { label: "侦察", value: "SCOUT" },
];

export const kpiCharacterFilterOptions = [
  { label: "钻机", value: "driller" },
  { label: "枪手", value: "gunner" },
  { label: "工程", value: "engineer" },
  { label: "侦察", value: "scout" },
  { label: "侦察-输出型", value: "scout_special" },
];

export function getFirstGroupInfo<T>(
  objectList: T[],
  compar: (a: T, b: T) => number,
  objKey: (a: T) => number,
  objLabel: (a: T) => string,
  otherPercentLimit: number,
): GroupInfo {
  objectList.sort(compar);

  let totalValue = 0.0;
  let countedValue = 0.0;

  for (const objectElement of objectList) {
    totalValue += objKey(objectElement);
  }

  const labelList: string[] = [];
  const dataList: number[] = [];

  for (const objectElement of objectList) {
    const countedPercent = countedValue / totalValue;
    if (countedPercent >= 1 - otherPercentLimit) {
      break;
    }

    labelList.push(objLabel(objectElement));
    dataList.push(objKey(objectElement));
    countedValue += objKey(objectElement);
  }

  labelList.push("其它");
  dataList.push(totalValue - countedValue);

  return {
    labelList,
    dataList,
  };
}

export function getKPICharacterOrder(source: string) {
  switch (source) {
    case "driller":
      return 0;
    case "engineer":
      return 1;
    case "gunner":
      return 2;
    case "scout":
      return 3;
    case "scout_special":
      return 4;
    default:
      return 5;
  }
}
