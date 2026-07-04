export interface Download {
  id: string;
  name: string;
  url: string;
  status: 'pending' | 'downloading' | 'paused' | 'completed' | 'error';
  progress: number;
  size: number;
  downloaded: number;
  speed: number;
  createdAt: Date;
  completedAt?: Date;
}

export interface DownloadHistory {
  id: string;
  name: string;
  url: string;
  status: 'completed' | 'error';
  size: number;
  completedAt: Date;
}

export type ViewType = 'downloads' | 'history' | 'settings';