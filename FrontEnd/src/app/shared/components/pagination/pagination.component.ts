import { Component, input, output, computed, CUSTOM_ELEMENTS_SCHEMA } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-pagination',
  standalone: true,
  imports: [CommonModule],
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  template: `
    <div class="flex items-center justify-between mt-4 pt-4 border-t border-white/5">
      <span class="text-xs text-neutral-500">
        {{ startItem() }}-{{ endItem() }} sur {{ totalItems() }}
      </span>
      <div class="flex items-center gap-2">
        <button (click)="onPrev()"
          [disabled]="currentPage() <= 1"
          class="px-3 py-1.5 text-xs font-medium rounded-lg border border-white/10 text-neutral-400 hover:text-white hover:bg-white/5 transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
          <iconify-icon icon="mdi:chevron-left" width="16"></iconify-icon>
        </button>
        @for (p of pages(); track p) {
          <button (click)="pageChange.emit(p)"
            class="px-3 py-1.5 text-xs font-medium rounded-lg transition-colors"
            [class]="p === currentPage() ? 'bg-teal-400/20 text-teal-400 border border-teal-400/30' : 'text-neutral-400 hover:text-white hover:bg-white/5 border border-white/10'">
            {{ p }}
          </button>
        }
        <button (click)="onNext()"
          [disabled]="currentPage() >= totalPages()"
          class="px-3 py-1.5 text-xs font-medium rounded-lg border border-white/10 text-neutral-400 hover:text-white hover:bg-white/5 transition-colors disabled:opacity-30 disabled:cursor-not-allowed">
          <iconify-icon icon="mdi:chevron-right" width="16"></iconify-icon>
        </button>
      </div>
    </div>
  `,
})
export class PaginationComponent {
  currentPage = input.required<number>();
  totalItems = input.required<number>();
  pageSize = input<number>(10);
  pageChange = output<number>();

  totalPages = computed(() => Math.ceil(this.totalItems() / this.pageSize()));
  startItem = computed(() => (this.currentPage() - 1) * this.pageSize() + 1);
  endItem = computed(() => Math.min(this.currentPage() * this.pageSize(), this.totalItems()));

  pages = computed(() => {
    const total = this.totalPages();
    const current = this.currentPage();
    const pages: number[] = [];
    const start = Math.max(1, current - 2);
    const end = Math.min(total, start + 4);
    for (let i = start; i <= end; i++) pages.push(i);
    return pages;
  });

  onPrev(): void {
    if (this.currentPage() > 1) this.pageChange.emit(this.currentPage() - 1);
  }

  onNext(): void {
    if (this.currentPage() < this.totalPages()) this.pageChange.emit(this.currentPage() + 1);
  }
}
