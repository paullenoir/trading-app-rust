import {
  Component,
  inject,
  signal,
  computed,
  OnInit,
  DestroyRef,
  CUSTOM_ELEMENTS_SCHEMA,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { forkJoin } from 'rxjs';
import { NavbarComponent } from '../../shared/components/navbar/navbar.component';

import { EmptyStateComponent } from '../../shared/components/empty-state/empty-state.component';
import { LoadingSkeletonComponent } from '../../shared/components/loading-skeleton/loading-skeleton.component';
import { PaginationComponent } from '../../shared/components/pagination/pagination.component';
import { WalletService } from '../../core/services/wallet.service';
import { TradeService } from '../../core/services/trade.service';
import { CalculationService } from '../../core/services/calculation.service';
import { WalletBalance, WalletHistoryItem } from '../../core/models/wallet.model';
import { ClosedTrade } from '../../core/models/trade.model';

type TransactionType = 'ajout' | 'retrait';
type FilterOption = 'Tout' | 'D\u00e9p\u00f4ts' | 'Retraits' | 'Gains Positifs' | 'Gains N\u00e9gatifs';

@Component({
  selector: 'app-wallet',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    NavbarComponent,
    EmptyStateComponent,
    LoadingSkeletonComponent,
    PaginationComponent,
  ],
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  templateUrl: './wallet.component.html',
})
export class WalletComponent implements OnInit {
  private readonly destroyRef = inject(DestroyRef);
  private readonly walletService = inject(WalletService);
  private readonly tradeService = inject(TradeService);
  private readonly calcService = inject(CalculationService);

  // State signals
  loading = signal(true);
  balance = signal<WalletBalance | null>(null);
  history = signal<WalletHistoryItem[]>([]);
  closedTrades = signal<ClosedTrade[]>([]);

  // Form signals
  transactionType = signal<TransactionType>('ajout');
  amount = signal(0);
  currency = signal('USD');
  transactionDate = signal('');

  // Filter & pagination signals
  activeFilter = signal<FilterOption>('Tout');
  currentPage = signal(1);
  pageSize = signal(10);

  // Filter options
  filterOptions: FilterOption[] = ['Tout', 'D\u00e9p\u00f4ts', 'Retraits', 'Gains Positifs', 'Gains N\u00e9gatifs'];

  // Computed: KPI values
  totalDeposited = computed(() => this.calcService.totalDeposited(this.history()));

  totalWithdrawn = computed(() => this.calcService.totalWithdrawn(this.history()));

  realizedGains = computed(() => this.calcService.totalProfits(this.closedTrades()));

  realizedLosses = computed(() => this.calcService.totalLosses(this.closedTrades()));

  roiGlobal = computed(() => {
    const deposited = this.totalDeposited();
    const gains = this.realizedGains();
    const losses = this.realizedLosses();
    return this.calcService.roi(gains, losses, deposited);
  });

  // Computed: merge history + closed trades into a unified list, then filter
  filteredHistory = computed(() => {
    const filter = this.activeFilter();
    const historyItems = this.history();
    const trades = this.closedTrades();

    // Build unified list
    type UnifiedItem = {
      type: 'deposit' | 'withdrawal' | 'gain_positive' | 'gain_negative';
      symbol?: string;
      date: string;
      amount: number;
      currency: string;
      percentage?: number;
    };

    const items: UnifiedItem[] = [];

    // Add history (deposits/withdrawals)
    for (const h of historyItems) {
      items.push({
        type: h.action === 'ajout' ? 'deposit' : 'withdrawal',
        date: h.date,
        amount: h.amount,
        currency: h.currency,
      });
    }

    // Add closed trades (gains)
    for (const t of trades) {
      items.push({
        type: t.gain_dollars >= 0 ? 'gain_positive' : 'gain_negative',
        symbol: t.symbol,
        date: t.date_vente || '',
        amount: t.gain_dollars,
        currency: t.devise,
        percentage: t.pourcentage_gain,
      });
    }

    // Sort by date descending
    items.sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime());

    // Filter
    switch (filter) {
      case 'D\u00e9p\u00f4ts':
        return items.filter((i) => i.type === 'deposit');
      case 'Retraits':
        return items.filter((i) => i.type === 'withdrawal');
      case 'Gains Positifs':
        return items.filter((i) => i.type === 'gain_positive');
      case 'Gains N\u00e9gatifs':
        return items.filter((i) => i.type === 'gain_negative');
      default:
        return items;
    }
  });

  // Computed: client-side pagination
  pagedHistory = computed(() => {
    const all = this.filteredHistory();
    const page = this.currentPage();
    const size = this.pageSize();
    const start = (page - 1) * size;
    return all.slice(start, start + size);
  });

  ngOnInit(): void {
    this.loadData();
  }

  private loadData(): void {
    forkJoin({
      balance: this.walletService.getBalance(),
      history: this.walletService.getHistory(),
      closedTrades: this.tradeService.getClosedTrades(),
    })
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: ({ balance, history, closedTrades }) => {
          this.balance.set(balance);
          this.history.set(history);
          this.closedTrades.set(closedTrades);
          this.loading.set(false);
        },
        error: () => this.loading.set(false),
      });
  }

  submitTransaction(): void {
    const req = {
      date: this.transactionDate(),
      action: this.transactionType(),
      amount: this.amount(),
      currency: this.currency(),
    };

    this.walletService
      .addTransaction(req)
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: () => {
          // Reset form
          this.amount.set(0);
          this.transactionDate.set('');
          // Reload data
          this.loading.set(true);
          this.loadData();
        },
      });
  }

  setFilter(filter: FilterOption): void {
    this.activeFilter.set(filter);
    this.currentPage.set(1);
  }

  onPageChange(page: number): void {
    this.currentPage.set(page);
  }

  formatCurrency(n: number): string {
    const prefix = n >= 0 ? '+' : '';
    return `${prefix}$${Math.abs(n).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 0 })}`;
  }

  formatCurrencyFull(n: number): string {
    const prefix = n >= 0 ? '+' : '-';
    return `${prefix}$${Math.abs(n).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
  }

  formatDate(dateStr: string): string {
    if (!dateStr) return '';
    const d = new Date(dateStr);
    const day = d.getDate().toString().padStart(2, '0');
    const month = (d.getMonth() + 1).toString().padStart(2, '0');
    const year = d.getFullYear();
    const hours = d.getHours().toString().padStart(2, '0');
    const minutes = d.getMinutes().toString().padStart(2, '0');
    return `${day}/${month}/${year} ${hours}:${minutes}`;
  }

  formatPercent(n: number): string {
    const prefix = n >= 0 ? '+' : '';
    return `${prefix}${n.toFixed(2)}%`;
  }
}
