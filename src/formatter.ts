import { format, formatRelative } from "date-fns";
import { zhCN } from "date-fns/locale";

export function formatMissionDate(timeStamp: number | undefined) {
  if (timeStamp !== undefined) {
    return format(new Date((timeStamp - 8 * 3600) * 1000), "yyyy-M-d HH:mm", { locale: zhCN });
  } else {
    return "";
  }
}

export function formatMissionDateRelative(timeStamp: number | undefined) {
  if (timeStamp !== undefined) {
    return formatRelative(new Date((timeStamp - 8 * 3600) * 1000), new Date(), { locale: zhCN });
  } else {
    return "";
  }
}

export function formatMissionTime(time: number | undefined) {
  if (time !== undefined) {
    time = Math.trunc(time);
    const hours = Math.floor(time / 3600);
    const minutes = Math.floor((time % 3600) / 60);
    const seconds = time % 60;

    if (time >= 3600) {
      return `${hours}小时${minutes}分${seconds}秒`;
    } else if (time >= 60) {
      return `${minutes}分${seconds}秒`;
    } else {
      return `${seconds}秒`;
    }
  } else {
    return "";
  }
}

export function formatMissionResult(result: number | undefined) {
  if (result !== undefined) {
    switch (result) {
      case 0:
        return "已完成";
      case 1:
        return "失败";
      case 2:
        return "放弃";
      default:
        return "未知";
    }
  } else {
    return "";
  }
}

export function formatMissionDifficulty(difficulty: number | undefined) {
  if (difficulty !== undefined) {
    switch (difficulty) {
      case 1:
        return "一级风险";
      case 2:
        return "二级风险";
      case 3:
        return "三级风险";
      case 4:
        return "四级风险";
      case 5:
        return "五级风险";
      case 100:
        return "深潜 阶段一 3";
      case 101:
        return "深潜 阶段二 3.5";
      case 102:
        return "深潜 阶段三 3.5";
      case 103:
        return "精英深潜 阶段一 4.5";
      case 104:
        return "精英深潜 阶段二 5";
      case 105:
        return "精英深潜 阶段三 5.5";
      default:
        return "未知";
    }
  } else {
    return "";
  }
}

export function formatCharacterTitle(promotionTimes: number | undefined) {
  if (promotionTimes !== undefined) {
    switch (promotionTimes) {
      case 0:
        return "None";
      case 1:
        return "Bronze I";
      case 2:
        return "Bronze II";
      case 3:
        return "Bronze III";
      case 4:
        return "Silver I";
      case 5:
        return "Silver II";
      case 6:
        return "Silver III";
      case 7:
        return "Gold I";
      case 8:
        return "Gold II";
      case 9:
        return "Gold III";
      case 10:
        return "Platinum I";
      case 11:
        return "Platinum II";
      case 12:
        return "Platinum III";
      case 13:
        return "Diamond I";
      case 14:
        return "Diamond II";
      case 15:
        return "Diamond III";
      case 16:
        return "Legendary I";
      case 17:
        return "Legendary II";
      case 18:
        return "Legendary III";
      default:
        return `Legendary IV(${promotionTimes - 18})`;
    }
  } else {
    return "";
  }
}

export function formatPercent(value: number | undefined) {
  if (value !== undefined) {
    return `${(value * 100).toFixed(1)}%`;
  } else {
    return "";
  }
}

export function generateCharacterClass(heroGameId: string) {
  switch (heroGameId) {
    case "DRILLER":
      return "driller";
    case "ENGINEER":
      return "engineer";
    case "GUNNER":
      return "gunner";
    case "SCOUT":
      return "scout";
    default:
      return "";
  }
}

export function generateTitleClass(promotionTimes: number | undefined) {
  if (promotionTimes == undefined) {
    return "";
  }
  if (promotionTimes == 0) {
    return "";
  } else if (promotionTimes >= 1 && promotionTimes <= 3) {
    return "bronze";
  } else if (promotionTimes >= 4 && promotionTimes <= 6) {
    return "silver";
  } else if (promotionTimes >= 7 && promotionTimes <= 9) {
    return "gold";
  } else if (promotionTimes >= 10 && promotionTimes <= 12) {
    return "platinum";
  } else if (promotionTimes >= 13 && promotionTimes <= 15) {
    return "diamond";
  } else {
    return "legendary";
  }
}

export function nFormatter(num: number | undefined, digits: number) {
  const lookup = [
    { value: 1, symbol: "" },
    { value: 1e3, symbol: "K" },
    { value: 1e6, symbol: "M" },
    { value: 1e9, symbol: "G" },
    { value: 1e12, symbol: "T" },
    { value: 1e15, symbol: "P" },
    { value: 1e18, symbol: "E" },
  ];

  if (num === undefined) {
    return "";
  }

  const regexp = /\.0+$|(?<=\.[0-9]*[1-9])0+$/;
  lookup.reverse();
  const item = lookup.find((item) => num >= item.value);
  return item ? (num / item.value).toFixed(digits).replace(regexp, "").concat(item.symbol) : "0";
}

export function generateKPICharacterClass(source: string) {
  switch (source) {
    case "driller":
      return "driller";
    case "engineer":
      return "engineer";
    case "gunner":
      return "gunner";
    case "scout":
      return "scout";
    case "scout_special":
      return "scout";
    default:
      return "";
  }
}

export function getKPICharacterName(source: string) {
  switch (source) {
    case "driller":
      return "钻机";
    case "engineer":
      return "工程";
    case "gunner":
      return "枪手";
    case "scout":
      return "侦察";
    case "scout_special":
      return "侦察-输出型";
    default:
      return source;
  }
}
