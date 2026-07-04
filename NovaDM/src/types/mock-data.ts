import { Download } from './index';

export const mockDownloads: Download[] = [
  {
    id: '1',
    name: 'Ubuntu.iso',
    url: 'https://releases.ubuntu.com/22.04/ubuntu-22.04.3-desktop-amd64.iso',
    status: 'downloading',
    progress: 43,
    size: 5368709120, // 5 GB
    downloaded: 2306867200, // 2.15 GB
    speed: 13003413, // 12.4 MB/s
    createdAt: new Date('2024-01-15T10:30:00'),
  },
  {
    id: '2',
    name: 'Movie.mkv',
    url: 'https://example.com/downloads/movie.mkv',
    status: 'paused',
    progress: 78,
    size: 1073741824, // 1 GB
    downloaded: 838860800, // 800 MB
    speed: 0,
    createdAt: new Date('2024-01-15T09:15:00'),
  },
  {
    id: '3',
    name: 'RustBook.pdf',
    url: 'https://example.com/books/rust-programming-language.pdf',
    status: 'completed',
    progress: 100,
    size: 52428800, // 50 MB
    downloaded: 52428800,
    speed: 0,
    createdAt: new Date('2024-01-14T14:20:00'),
    completedAt: new Date('2024-01-14T14:25:00'),
  },
];