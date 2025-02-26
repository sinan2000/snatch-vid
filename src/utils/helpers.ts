export function parseDownloadPercentage(line: string) {
  const regex = /\[download\]\s+([\d.]+)%/;
  const match = regex.exec(line);
  if (match && match[1]) {
    const percent = parseFloat(match[1]);
    return percent;
  }

  return null;
}