export type NoticeKind = 'error' | 'success' | 'info';

export interface Notice {
  id: number;
  kind: NoticeKind;
  message: string;
}

const LIFETIME_MS = { error: 9000, success: 3500, info: 5000 } as const;

/** Short messages shown over the interface for something that happened outside a form. */
class Notices {
  items = $state<Notice[]>([]);
  private next = 1;

  show(kind: NoticeKind, message: string): void {
    const id = this.next++;
    this.items.push({ id, kind, message });
    setTimeout(() => this.dismiss(id), LIFETIME_MS[kind]);
  }

  error(message: string): void {
    this.show('error', message);
  }

  success(message: string): void {
    this.show('success', message);
  }

  dismiss(id: number): void {
    this.items = this.items.filter((notice) => notice.id !== id);
  }
}

export const notices = new Notices();
