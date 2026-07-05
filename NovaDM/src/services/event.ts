// Event service for listening to Tauri events
// Only this service may listen to Tauri events

import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface DownloadProgressData {
  id: string;
  progress: number | null;
  downloaded_bytes: number;
  total_bytes: number | null;
  speed: number;
  status: string;
}

export interface DownloadCompletedData {
  id: string;
}

export interface DownloadErrorData {
  id: string;
  message: string;
}

export interface DownloadCancelledData {
  id: string;
}

export interface DownloadQueuedData {
  id: string;
  position: number;
}

export interface DownloadStartedData {
  id: string;
}

export interface DownloadRetryData {
  id: string;
  new_id: string;
}

type ProgressCallback = (data: DownloadProgressData) => void;
type CompletedCallback = (data: DownloadCompletedData) => void;
type ErrorCallback = (data: DownloadErrorData) => void;
type CancelledCallback = (data: DownloadCancelledData) => void;
type QueuedCallback = (data: DownloadQueuedData) => void;
type StartedCallback = (data: DownloadStartedData) => void;
type RetryCallback = (data: DownloadRetryData) => void;

class EventService {
  private progressUnlistenPromise: Promise<UnlistenFn> | null = null;
  private completedUnlistenPromise: Promise<UnlistenFn> | null = null;
  private errorUnlistenPromise: Promise<UnlistenFn> | null = null;
  private cancelledUnlistenPromise: Promise<UnlistenFn> | null = null;
  private queuedUnlistenPromise: Promise<UnlistenFn> | null = null;
  private startedUnlistenPromise: Promise<UnlistenFn> | null = null;
  private retryUnlistenPromise: Promise<UnlistenFn> | null = null;
  private progressCallbacks: Set<ProgressCallback> = new Set();
  private completedCallbacks: Set<CompletedCallback> = new Set();
  private errorCallbacks: Set<ErrorCallback> = new Set();
  private cancelledCallbacks: Set<CancelledCallback> = new Set();
  private queuedCallbacks: Set<QueuedCallback> = new Set();
  private startedCallbacks: Set<StartedCallback> = new Set();
  private retryCallbacks: Set<RetryCallback> = new Set();

  /**
   * Register a listener for download progress events
   * @param callback - Function to call when progress event is received
   * @returns Unlisten function to remove the listener
   */
  registerProgressListener(callback: ProgressCallback): UnlistenFn {
    this.progressCallbacks.add(callback);

    if (!this.progressUnlistenPromise) {
      this.progressUnlistenPromise = listen<DownloadProgressData>(
        'download://progress',
        (event) => {
          this.progressCallbacks.forEach((cb) => cb(event.payload));
        }
      );
    }

    return () => {
      this.progressCallbacks.delete(callback);
    };
  }

  /**
   * Register a listener for download completed events
   * @param callback - Function to call when completed event is received
   * @returns Unlisten function to remove the listener
   */
  registerCompletedListener(callback: CompletedCallback): UnlistenFn {
    this.completedCallbacks.add(callback);

    if (!this.completedUnlistenPromise) {
      this.completedUnlistenPromise = listen<DownloadCompletedData>(
        'download://completed',
        (event) => {
          this.completedCallbacks.forEach((cb) => cb(event.payload));
        }
      );
    }

    return () => {
      this.completedCallbacks.delete(callback);
    };
  }

  /**
   * Register a listener for download error events
   * @param callback - Function to call when error event is received
   * @returns Unlisten function to remove the listener
   */
  registerErrorListener(callback: ErrorCallback): UnlistenFn {
    this.errorCallbacks.add(callback);

    if (!this.errorUnlistenPromise) {
      this.errorUnlistenPromise = listen<DownloadErrorData>(
        'download://error',
        (event) => {
          this.errorCallbacks.forEach((cb) => cb(event.payload));
        }
      );
    }

    return () => {
      this.errorCallbacks.delete(callback);
    };
  }

  /**
   * Register a listener for download queued events
   * @param callback - Function to call when queued event is received
   * @returns Unlisten function to remove the listener
   */
  registerQueuedListener(callback: QueuedCallback): UnlistenFn {
    this.queuedCallbacks.add(callback);

    if (!this.queuedUnlistenPromise) {
      this.queuedUnlistenPromise = listen<DownloadQueuedData>(
        'download://queued',
        (event) => {
          this.queuedCallbacks.forEach((cb) => cb(event.payload));
        }
      );
    }

    return () => {
      this.queuedCallbacks.delete(callback);
    };
  }

  /**
   * Register a listener for download started events
   * @param callback - Function to call when started event is received
   * @returns Unlisten function to remove the listener
   */
  registerStartedListener(callback: StartedCallback): UnlistenFn {
    this.startedCallbacks.add(callback);

    if (!this.startedUnlistenPromise) {
      this.startedUnlistenPromise = listen<DownloadStartedData>(
        'download://started',
        (event) => {
          this.startedCallbacks.forEach((cb) => cb(event.payload));
        }
      );
    }

    return () => {
      this.startedCallbacks.delete(callback);
    };
  }

  /**
   * Register a listener for download retry events
   * @param callback - Function to call when retry event is received
   * @returns Unlisten function to remove the listener
   */
  registerRetryListener(callback: RetryCallback): UnlistenFn {
    this.retryCallbacks.add(callback);

    if (!this.retryUnlistenPromise) {
      this.retryUnlistenPromise = listen<DownloadRetryData>(
        'download://retry',
        (event) => {
          this.retryCallbacks.forEach((cb) => cb(event.payload));
        }
      );
    }

    return () => {
      this.retryCallbacks.delete(callback);
    };
  }

  /**
   * Register a listener for download cancelled events
   * @param callback - Function to call when cancelled event is received
   * @returns Unlisten function to remove the listener
   */
  registerCancelledListener(callback: CancelledCallback): UnlistenFn {
    this.cancelledCallbacks.add(callback);

    if (!this.cancelledUnlistenPromise) {
      this.cancelledUnlistenPromise = listen<DownloadCancelledData>(
        'download://cancelled',
        (event) => {
          this.cancelledCallbacks.forEach((cb) => cb(event.payload));
        }
      );
    }

    return () => {
      this.cancelledCallbacks.delete(callback);
    };
  }

  /**
   * Unregister all listeners
   */
  async unregisterAll(): Promise<void> {
    this.progressCallbacks.clear();
    this.completedCallbacks.clear();
    this.errorCallbacks.clear();
    this.cancelledCallbacks.clear();
    this.queuedCallbacks.clear();
    this.startedCallbacks.clear();
    this.retryCallbacks.clear();

    if (this.progressUnlistenPromise) {
      const unlisten = await this.progressUnlistenPromise;
      unlisten();
      this.progressUnlistenPromise = null;
    }

    if (this.completedUnlistenPromise) {
      const unlisten = await this.completedUnlistenPromise;
      unlisten();
      this.completedUnlistenPromise = null;
    }

    if (this.errorUnlistenPromise) {
      const unlisten = await this.errorUnlistenPromise;
      unlisten();
      this.errorUnlistenPromise = null;
    }

    if (this.queuedUnlistenPromise) {
      const unlisten = await this.queuedUnlistenPromise;
      unlisten();
      this.queuedUnlistenPromise = null;
    }

    if (this.startedUnlistenPromise) {
      const unlisten = await this.startedUnlistenPromise;
      unlisten();
      this.startedUnlistenPromise = null;
    }

    if (this.retryUnlistenPromise) {
      const unlisten = await this.retryUnlistenPromise;
      unlisten();
      this.retryUnlistenPromise = null;
    }
  }
}

// Export singleton instance
export const eventService = new EventService();