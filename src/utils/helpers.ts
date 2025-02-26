export function parseDownloadPercentage(line: string) {
  const regex = /\[download\]\s+([\d.]+)%/;
  const match = regex.exec(line);
  if (match && match[1]) {
    const percent = parseFloat(match[1]);
    return percent;
  }

  return null;
}

export function parseDownloadingItem(line: string) {
  const regex = /\[download\]\s+Downloading item\s+(\d+)\s+of\s+(\d+)/i;
  const match = regex.exec(line);
  if (match && match[1] && match[2]) {
    return {
      current: parseInt(match[1], 10),
      total: parseInt(match[2], 10)
    };
  }
  return null;
}