import { Component, inject, signal, computed, OnInit, DestroyRef, CUSTOM_ELEMENTS_SCHEMA } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { forkJoin } from 'rxjs';
import { NavbarComponent } from '../../shared/components/navbar/navbar.component';
import { KpiCardComponent } from '../../shared/components/kpi-card/kpi-card.component';
import { StrategySpheresComponent } from '../../shared/components/strategy-spheres/strategy-spheres.component';
import { EmptyStateComponent } from '../../shared/components/empty-state/empty-state.component';
import { LoadingSkeletonComponent } from '../../shared/components/loading-skeleton/loading-skeleton.component';
import { WalletService } from '../../core/services/wallet.service';
import { TradeService } from '../../core/services/trade.service';
import { StockService } from '../../core/services/stock.service';
import { CalculationService } from '../../core/services/calculation.service';
import { AuthService } from '../../core/services/auth.service';
import { PositionWithGain, ConsensusType } from '../../core/models/trade.model';
import { StockWithStrategies } from '../../core/models/stock.model';
import { FormsModule } from '@angular/forms';

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [
    CommonModule, RouterLink, FormsModule,
    NavbarComponent, KpiCardComponent, StrategySpheresComponent,
    EmptyStateComponent, LoadingSkeletonComponent,
  ],
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  templateUrl: './dashboard.component.html',
})
export class DashboardComponent implements OnInit {
  private readonly destroyRef = inject(DestroyRef);
  private readonly walletService = inject(WalletService);
  private readonly tradeService = inject(TradeService);
  private readonly stockService = inject(StockService);
  private readonly calcService = inject(CalculationService);
  private readonly authService = inject(AuthService);

  loading = signal(true);
  portfolioTotal = signal(0);
  tresorerie = signal(0);
  totalGain = signal(0);
  totalGainPercent = signal(0);
  openCount = signal(0);
  avgHoldingTime = signal(0);
  positivePositions = signal<PositionWithGain[]>([]);
  negativePositions = signal<PositionWithGain[]>([]);
  topAchats = signal<StockWithStrategies[]>([]);
  watchlist = signal<StockWithStrategies[]>([]);
  watchlistSearch = signal('');
  timeFilter = signal('1A');
  showDepositModal = signal(false);
  showWithdrawModal = signal(false);

  totalPositiveGain = computed(() =>
    this.positivePositions().reduce((sum, p) => sum + p.gainDollars, 0)
  );
  totalNegativeGain = computed(() =>
    this.negativePositions().reduce((sum, p) => sum + p.gainDollars, 0)
  );
  filteredWatchlist = computed(() => {
    const search = this.watchlistSearch().toLowerCase();
    if (!search) return this.watchlist();
    return this.watchlist().filter(
      (s) => s.stock.symbol.toLowerCase().includes(search) || s.stock.name.toLowerCase().includes(search)
    );
  });

  ngOnInit(): void {
    this.loadData();
  }

  private loadData(): void {
    forkJoin({
      balance: this.walletService.getBalance(),
      openWithRec: this.tradeService.getOpenTradesWithRecommendations(),
      closed: this.tradeService.getClosedTrades(),
      stocks: this.stockService.getStocksWithStrategies(),
    })
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: ({ balance, openWithRec, closed, stocks }) => {
          this.portfolioTotal.set(balance.portfolio_total);
          this.tresorerie.set(balance.tresorerie);

          const enriched = this.calcService.enrichPositions(openWithRec);
          this.positivePositions.set(
            enriched.filter((p) => p.gainPercent >= 0).sort((a, b) => b.gainPercent - a.gainPercent)
          );
          this.negativePositions.set(
            enriched.filter((p) => p.gainPercent < 0).sort((a, b) => a.gainPercent - b.gainPercent)
          );
          this.openCount.set(openWithRec.length);

          this.totalGain.set(this.calcService.totalGain(closed));
          this.avgHoldingTime.set(this.calcService.avgHoldingTime(closed));

          const totalInvested = openWithRec.reduce((s, t) => s + t.prix_moyen * t.quantite_totale, 0);
          this.totalGainPercent.set(totalInvested > 0 ? (this.totalGain() / totalInvested) * 100 : 0);

          // Top achats: stocks with consensus Buy/Strong Buy that are NOT in open positions
          const openSymbols = new Set(openWithRec.map((t) => t.symbol));
          this.topAchats.set(
            stocks
              .filter((s) => {
                if (openSymbols.has(s.stock.symbol)) return false;
                const consensus = this.calcService.calculateConsensus(s.strategies);
                return consensus === 'Buy' || consensus === 'Strong Buy';
              })
              .slice(0, 6)
          );
          this.watchlist.set(stocks.slice(0, 10));

          this.loading.set(false);
        },
        error: () => this.loading.set(false),
      });
  }

  getBadgeClass(consensus: ConsensusType): string {
    switch (consensus) {
      case 'Strong Buy': return 'badge-mini badge-strong-buy';
      case 'Buy': return 'badge-mini badge-buy';
      case 'Hold': return 'badge-mini badge-hold';
      case 'Sell': return 'badge-mini badge-sell';
      case 'Strong Sell': return 'badge-mini badge-strong-sell';
    }
  }

  getConsensusForStock(stock: StockWithStrategies): ConsensusType {
    return this.calcService.calculateConsensus(stock.strategies);
  }

  formatCurrency(n: number): string {
    return n.toLocaleString('en-US', { style: 'currency', currency: 'USD', minimumFractionDigits: 0, maximumFractionDigits: 0 });
  }

  formatCurrencyFull(n: number): string {
    return n.toLocaleString('en-US', { style: 'currency', currency: 'USD', minimumFractionDigits: 2 });
  }

  formatPercent(n: number): string {
    const prefix = n >= 0 ? '+' : '';
    return `${prefix}${n.toFixed(2)}%`;
  }

  timeFilters = ['1J', '1S', '1M', '1A', 'Tout'];
}
