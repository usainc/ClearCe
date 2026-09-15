import { number } from "../i18n";
import type { ImageItem, Job } from "../types";
export const samples: ImageItem[] = [
  {
    id: "sample-portrait",
    name: "portrait_01.png",
    src: "/assets/portrait.png",
    width: 1536,
    height: 1024,
    bytes: 2371373,
    demo: true,
  },
];
export const demoJobs: Job[] = [
  "portrait_01.png",
  "weekend_portrait.png",
  "studio_detail.png",
  "warm_light.png",
  "portrait_closeup.png",
  "window_light.png",
].map((name, i) => ({
  id: `demo-${i}`,
  image: { ...samples[0], id: `demo-image-${i}`, name },
  mode: i === 1 ? "Portrait" : "Photo",
  scale: i % 2 ? 2 : 4,
  format: i % 2 ? "JPG" : "PNG",
  status: i === 4 ? "Failed" : "Completed",
  progress: i === 4 ? 0 : 100,
  seconds: [14.2, 8.6, 16.1, 13.8, 0, 12][i],
  date: `2026-09-${String(13 - i).padStart(2, "0")}T10:42:00`,
}));
export function formatBytes(bytes: number) {
  return bytes >= 1e6
    ? `${number(bytes / 1e6, 1)} MB`
    : `${number(bytes / 1000)} KB`;
}
