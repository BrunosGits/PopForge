export type Mode = 'convert' | 'extract';
export type JobStatus = 'pending' | 'running' | 'done' | 'error';

export type ConversionResult = {
  success: boolean;
  message: string;
  output_path: string | null;
  command_preview: string | null;
};

export type ConversionProgress = {
  current: number;
  total: number;
  fileName: string;
  stage: string;
  filePercent: number | null;
};

export type GameMetadata = {
  serial: string;
  title: string;
  region: string;
  coverPath: string | null;
  source: string;
  cached: boolean;
};

export type ConversionOptions = {
  mode: Mode;
  gameName: string;
  gameId: string;
  compression: number;
  outputTemplate: string;
  outputFolder: string;
  popstationPath: string;
  chdmanPath: string;
  icon0Path: string;
  pic0Path: string;
  pic1Path: string;
};

export type ToolStatus = {
  name: string;
  available: boolean;
  detail: string;
  path: string | null;
};

export type Job = {
  id: number;
  filePath: string;
  fileName: string;
  mode: Mode;
  status: JobStatus;
  message: string | null;
  outputPath: string | null;
  commandPreview: string | null;
  metadata: GameMetadata | null;
};

export type AppSettings = {
  lastOutputFolder: string;
};
