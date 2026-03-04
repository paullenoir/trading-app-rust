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
import { ActivatedRoute } from '@angular/router';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { forkJoin } from 'rxjs';
import { NgApexchartsModule } from 'ng-apexcharts';
import { NavbarComponent } from '../../shared/components/navbar/navbar.component';
import { EmptyStateComponent } from '../../shared/components/empty-state/empty-state.component';
import { LoadingSkeletonComponent } from '../../shared/components/loading-skeleton/loading-skeleton.component';
import { StockService } from '../../core/services/stock.service';
import { CalculationService } from '../../core/services/calculation.service';
import {
  StockWithStrategies,
  HistoricalDataPoint,
  IndicatorsResponse,
} from '../../core/models/stock.model';
import { Strategy, ConsensusType } from '../../core/models/trade.model';

import {
  ApexAxisChartSeries,
  ApexChart,
  ApexXAxis,
  ApexYAxis,
  ApexPlotOptions,
  ApexGrid,
  ApexTheme,
  ApexTooltip,
  ApexDataLabels,
  ApexStroke,
} from 'ng-apexcharts';

export interface ChartOptions {
  series: ApexAxisChartSeries;
  chart: ApexChart;
  xaxis: ApexXAxis;
  yaxis: ApexYAxis;
  plotOptions: ApexPlotOptions;
  grid: ApexGrid;
  theme: ApexTheme;
  tooltip: ApexTooltip;
  dataLabels: ApexDataLabels;
  stroke: ApexStroke;
}

export interface VolumeChartOptions {
  series: ApexAxisChartSeries;
  chart: ApexChart;
  xaxis: ApexXAxis;
  yaxis: ApexYAxis;
  plotOptions: ApexPlotOptions;
  grid: ApexGrid;
  theme: ApexTheme;
  tooltip: ApexTooltip;
  dataLabels: ApexDataLabels;
}

@Component({
  selector: 'app-chart',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    NgApexchartsModule,
    NavbarComponent,
    EmptyStateComponent,
    LoadingSkeletonComponent,
  ],
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  templateUrl: './chart.component.html',
})
export class ChartComponent implements OnInit {
  private readonly destroyRef = inject(DestroyRef);
  private readonly route = inject(ActivatedRoute);
  private readonly stockService = inject(StockService);
  private readonly calcService = inject(CalculationService);

  loading = signal(true);
  selectedSymbol = signal('AAPL');
  timeframe = signal('Daily');
  symbols = signal<string[]>([]);
  stocksMap = signal<Map<string, StockWithStrategies>>(new Map());
  historicalData = signal<HistoricalDataPoint[]>([]);
  indicators = signal<IndicatorsResponse | null>(null);
  strategies = signal<Strategy[]>([]);

  chartOptions = signal<ChartOptions>({
    series: [],
    chart: {
      type: 'candlestick',
      height: 450,
      background: 'transparent',
      toolbar: { show: false },
      zoom: { enabled: true },
    },
    xaxis: {
      type: 'datetime',
      labels: { style: { colors: '#737373' } },
      axisBorder: { show: false },
      axisTicks: { show: false },
    },
    yaxis: {
      tooltip: { enabled: true },
      labels: {
        style: { colors: '#737373' },
        formatter: (val: number) => '$' + val.toFixed(2),
      },
    },
    plotOptions: {
      candlestick: {
        colors: {
          upward: '#10b981',
          downward: '#9333ea',
        },
      },
    },
    grid: {
      borderColor: 'rgba(255,255,255,0.03)',
      strokeDashArray: 4,
    },
    theme: { mode: 'dark' },
    tooltip: {
      theme: 'dark',
    },
    dataLabels: { enabled: false },
    stroke: { width: 1 },
  });

  volumeChartOptions = signal<VolumeChartOptions>({
    series: [],
    chart: {
      type: 'bar',
      height: 120,
      background: 'transparent',
      toolbar: { show: false },
      sparkline: { enabled: false },
    },
    xaxis: {
      type: 'datetime',
      labels: { show: false },
      axisBorder: { show: false },
      axisTicks: { show: false },
    },
    yaxis: {
      labels: { show: false },
    },
    plotOptions: {
      bar: {
        columnWidth: '80%',
      },
    },
    grid: {
      show: false,
    },
    theme: { mode: 'dark' },
    tooltip: { enabled: false },
    dataLabels: { enabled: false },
  });

  consensus = computed<ConsensusType>(() => {
    return this.calcService.calculateConsensus(this.strategies());
  });

  consensusClass = computed(() => {
    const c = this.consensus();
    if (c === 'Strong Buy' || c === 'Buy') return '';
    if (c === 'Strong Sell' || c === 'Sell') return 'sell-consensus';
    return 'neutral-consensus';
  });

  consensusText = computed(() => {
    const c = this.consensus();
    if (c === 'Strong Buy' || c === 'Buy') return 'BUY';
    if (c === 'Strong Sell' || c === 'Sell') return 'SELL';
    return 'HOLD';
  });

  consensusColor = computed(() => {
    const c = this.consensus();
    if (c === 'Strong Buy' || c === 'Buy') return 'text-emerald-500';
    if (c === 'Strong Sell' || c === 'Sell') return 'text-rose-500';
    return 'text-neutral-400';
  });

  voteCounts = computed(() => {
    const strats = this.strategies();
    let buy = 0, sell = 0, hold = 0;
    for (const s of strats) {
      if (s.recommendation === 'BUY') buy++;
      else if (s.recommendation === 'SELL') sell++;
      else hold++;
    }
    return { buy, sell, hold };
  });

  consensusDescription = computed(() => {
    const votes = this.voteCounts();
    const total = votes.buy + votes.sell + votes.hold;
    const c = this.consensusText();
    if (c === 'BUY') return `${votes.buy} stratégie${votes.buy > 1 ? 's' : ''} sur ${total} recommande${votes.buy > 1 ? 'nt' : ''} l'achat`;
    if (c === 'SELL') return `${votes.sell} stratégie${votes.sell > 1 ? 's' : ''} sur ${total} recommande${votes.sell > 1 ? 'nt' : ''} la vente`;
    return `${votes.hold} stratégie${votes.hold > 1 ? 's' : ''} sur ${total} recommande${votes.hold > 1 ? 'nt' : ''} de maintenir`;
  });

  ohlc = computed(() => {
    const data = this.historicalData();
    if (data.length === 0) return { open: 0, high: 0, low: 0, close: 0, volume: 0 };
    const last = data[data.length - 1];
    return {
      open: last.open,
      high: last.high,
      low: last.low,
      close: last.close,
      volume: last.volume,
    };
  });

  isBullish = computed(() => {
    const o = this.ohlc();
    return o.close >= o.open;
  });

  currentStockName = computed(() => {
    const map = this.stocksMap();
    const stock = map.get(this.selectedSymbol());
    return stock ? stock.stock.name : '';
  });

  ngOnInit(): void {
    const routeSymbol = this.route.snapshot.paramMap.get('symbol');
    if (routeSymbol) {
      this.selectedSymbol.set(routeSymbol.toUpperCase());
    }
    this.loadStocksList();
  }

  private loadStocksList(): void {
    this.stockService
      .getStocksWithStrategies()
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: (stocks) => {
          const syms = stocks.map((s) => s.stock.symbol);
          this.symbols.set(syms);
          const map = new Map<string, StockWithStrategies>();
          for (const s of stocks) {
            map.set(s.stock.symbol, s);
          }
          this.stocksMap.set(map);

          const currentStock = map.get(this.selectedSymbol());
          if (currentStock) {
            this.strategies.set(currentStock.strategies);
          }

          this.loadChartData(this.selectedSymbol());
        },
        error: () => this.loading.set(false),
      });
  }

  loadChartData(symbol: string): void {
    this.loading.set(true);
    const tf = this.timeframe() === '4H' ? '4h' : '1d';

    forkJoin({
      historical: this.stockService.getHistorical(symbol, tf),
      indicators: this.stockService.getIndicators(symbol, tf),
    })
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: ({ historical, indicators }) => {
          this.historicalData.set(historical.data);
          this.indicators.set(indicators);
          this.buildChartOptions(historical.data);
          this.loading.set(false);
        },
        error: () => this.loading.set(false),
      });
  }

  onSymbolChange(): void {
    const currentStock = this.stocksMap().get(this.selectedSymbol());
    if (currentStock) {
      this.strategies.set(currentStock.strategies);
    }
    this.loadChartData(this.selectedSymbol());
  }

  onTimeframeChange(tf: string): void {
    this.timeframe.set(tf);
    this.loadChartData(this.selectedSymbol());
  }

  private buildChartOptions(data: HistoricalDataPoint[]): void {
    const candleData = data.map((d) => ({
      x: new Date(d.timestamp).getTime(),
      y: [d.open, d.high, d.low, d.close],
    }));

    const volumeData = data.map((d) => ({
      x: new Date(d.timestamp).getTime(),
      y: d.volume,
    }));

    const volumeColors = data.map((d) =>
      d.close >= d.open ? '#10b981' : '#9333ea'
    );

    this.chartOptions.update((opts) => ({
      ...opts,
      series: [{ name: 'Candlestick', data: candleData }],
    }));

    this.volumeChartOptions.update((opts) => ({
      ...opts,
      series: [{ name: 'Volume', data: volumeData }],
      plotOptions: {
        bar: {
          columnWidth: '80%',
          colors: {
            ranges: [
              { from: 0, to: Infinity, color: 'rgba(16,185,129,0.4)' },
            ],
          },
        },
      },
    }));
  }

  getStrategyBadgeClass(recommendation: string): string {
    switch (recommendation) {
      case 'BUY':
        return 'strategy-badge badge-buy';
      case 'SELL':
        return 'strategy-badge badge-sell';
      default:
        return 'strategy-badge badge-neutral';
    }
  }

  formatPrice(n: number): string {
    return '$' + n.toFixed(2);
  }

  formatNumber(n: number): string {
    return n.toFixed(2);
  }
}
