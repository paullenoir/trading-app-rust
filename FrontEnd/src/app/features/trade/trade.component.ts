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
import { NavbarComponent } from '../../shared/components/navbar/navbar.component';
import { EmptyStateComponent } from '../../shared/components/empty-state/empty-state.component';
import { LoadingSkeletonComponent } from '../../shared/components/loading-skeleton/loading-skeleton.component';
import { PaginationComponent } from '../../shared/components/pagination/pagination.component';
import { TradeService } from '../../core/services/trade.service';
import { CalculationService } from '../../core/services/calculation.service';
import { ClosedTrade } from '../../core/models/trade.model';

@Component({
  selector: 'app-trade',
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
  templateUrl: './trade.component.html',
})
export class TradeComponent implements OnInit {
  private readonly destroyRef = inject(DestroyRef);
  private readonly tradeService = inject(TradeService);
  private readonly calcService = inject(CalculationService);

  // State signals
  loading = signal(true);
  closedTrades = signal<ClosedTrade[]>([]);

  // Filter signals
  filterPeriod = signal<string>('Tout');
  filterPerformance = signal<string>('Tout');
  filterSymbol = signal<string>('');
  filterDevise = signal<string>('Tout');
  sortBy = signal<string>('Date (récent)');

  // Pagination signals
  currentPage = signal(1);
  pageSize = signal(25);

  // Filter options
  periodOptions = ['Tout', '7 jours', '30 jours', '90 jours', '1 an'];
  performanceOptions = ['Tout', 'Gains', 'Pertes'];
  deviseOptions = ['Tout', 'CAD', 'USD'];
  sortOptions = ['Date (récent)', 'Date (ancien)', 'Gain % (desc)', 'Gain % (asc)', 'Gain $ (desc)', 'Gain $ (asc)', 'Durée (desc)', 'Durée (asc)'];

  // Computed: filtered trades
  filteredTrades = computed(() => {
    let trades = this.closedTrades();

    // Period filter
    const period = this.filterPeriod();
    if (period !== 'Tout') {
      const now = new Date();
      let daysBack = 0;
      switch (period) {
        case '7 jours': daysBack = 7; break;
        case '30 jours': daysBack = 30; break;
        case '90 jours': daysBack = 90; break;
        case '1 an': daysBack = 365; break;
      }
      if (daysBack > 0) {
        const cutoff = new Date(now.getTime() - daysBack * 86400000);
        trades = trades.filter((t) => {
          if (!t.date_vente) return false;
          return new Date(t.date_vente) >= cutoff;
        });
      }
    }

    // Performance filter
    const perf = this.filterPerformance();
    if (perf === 'Gains') {
      trades = trades.filter((t) => t.gain_dollars >= 0);
    } else if (perf === 'Pertes') {
      trades = trades.filter((t) => t.gain_dollars < 0);
    }

    // Symbol filter
    const sym = this.filterSymbol().toUpperCase().trim();
    if (sym) {
      trades = trades.filter((t) => t.symbol.toUpperCase().includes(sym));
    }

    // Devise filter
    const devise = this.filterDevise();
    if (devise !== 'Tout') {
      trades = trades.filter((t) => t.devise === devise);
    }

    return trades;
  });

  // Computed: sorted trades
  sortedTrades = computed(() => {
    const trades = [...this.filteredTrades()];
    const sort = this.sortBy();

    switch (sort) {
      case 'Date (récent)':
        return trades.sort((a, b) => new Date(b.date_vente ?? '').getTime() - new Date(a.date_vente ?? '').getTime());
      case 'Date (ancien)':
        return trades.sort((a, b) => new Date(a.date_vente ?? '').getTime() - new Date(b.date_vente ?? '').getTime());
      case 'Gain % (desc)':
        return trades.sort((a, b) => b.pourcentage_gain - a.pourcentage_gain);
      case 'Gain % (asc)':
        return trades.sort((a, b) => a.pourcentage_gain - b.pourcentage_gain);
      case 'Gain $ (desc)':
        return trades.sort((a, b) => b.gain_dollars - a.gain_dollars);
      case 'Gain $ (asc)':
        return trades.sort((a, b) => a.gain_dollars - b.gain_dollars);
      case 'Durée (desc)':
        return trades.sort((a, b) => b.temps_jours - a.temps_jours);
      case 'Durée (asc)':
        return trades.sort((a, b) => a.temps_jours - b.temps_jours);
      default:
        return trades;
    }
  });

  // Computed: paged trades (client-side pagination)
  pagedTrades = computed(() => {
    const start = (this.currentPage() - 1) * this.pageSize();
    return this.sortedTrades().slice(start, start + this.pageSize());
  });

  // Computed: total results count
  totalResults = computed(() => this.filteredTrades().length);

  // Computed: win rate
  winRate = computed(() => {
    const trades = this.closedTrades();
    if (trades.length === 0) return 0;
    const wins = trades.filter((t) => t.gain_dollars > 0).length;
    return Math.round((wins / trades.length) * 100);
  });

  // Computed: total profits
  totalProfits = computed(() => this.calcService.totalGain(this.closedTrades()));

  // Computed: average holding time
  avgHoldingTime = computed(() => this.calcService.avgHoldingTime(this.closedTrades()));

  ngOnInit(): void {
    this.loadClosedTrades();
  }

  private loadClosedTrades(): void {
    this.tradeService
      .getClosedTrades()
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: (trades) => {
          this.closedTrades.set(trades);
          this.loading.set(false);
        },
        error: () => this.loading.set(false),
      });
  }

  onFilterChange(): void {
    this.currentPage.set(1);
  }

  onPageChange(page: number): void {
    this.currentPage.set(page);
  }

  onPageSizeChange(size: number): void {
    this.pageSize.set(size);
    this.currentPage.set(1);
  }

  exportCsv(): void {
    const trades = this.sortedTrades();
    if (trades.length === 0) return;

    const headers = ['Symbole', 'Date Achat', 'Devise', 'Prix Achat', 'Qté Achat', 'Date Vente', 'Prix Vente', 'Qté Vente', '% Gain', '$ Gain', 'Durée (jours)'];
    const rows = trades.map((t) => [
      t.symbol,
      t.date_achat ?? '',
      t.devise,
      t.prix_achat?.toFixed(2) ?? '',
      t.quantite_achat,
      t.date_vente ?? '',
      t.prix_vente?.toFixed(2) ?? '',
      t.quantite_vente,
      t.pourcentage_gain.toFixed(2),
      t.gain_dollars.toFixed(2),
      t.temps_jours,
    ]);

    const csv = [headers.join(','), ...rows.map((r) => r.join(','))].join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'trades_export.csv';
    a.click();
    URL.revokeObjectURL(url);
  }

  formatCurrency(n: number): string {
    return n.toLocaleString('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
    });
  }

  formatPercent(n: number): string {
    const prefix = n >= 0 ? '+' : '';
    return `${prefix}${n.toFixed(2)}%`;
  }

  formatDate(date: string | undefined): string {
    if (!date) return '—';
    const d = new Date(date);
    return d.toLocaleDateString('fr-CA', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
    });
  }

  isGain(trade: ClosedTrade): boolean {
    return trade.gain_dollars >= 0;
  }
}
