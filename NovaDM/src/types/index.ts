export interface Download {
  id: string;
  name: string;
  url: string;
  status: 'pending' | 'downloading' | 'paused' | 'completed' | 'error' | 'cancelled';
  progress: number;
  size: number;
  downloaded: number;
  speed: number;
  createdAt: Date;
  completedAt?: Date;
  error?: string;
  queuePosition?: number;
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