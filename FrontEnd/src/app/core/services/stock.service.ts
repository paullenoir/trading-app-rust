import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';
import {
  StockWithStrategies,
  HistoricalResponse,
  IndicatorsResponse,
} from '../models/stock.model';

@Injectable({ providedIn: 'root' })
export class StockService {
  private readonly api = environment.apiBaseUrl;

  constructor(private readonly http: HttpClient) {}

  getStocksWithStrategies(): Observable<StockWithStrategies[]> {
    return this.http.get<StockWithStrategies[]>(`${this.api}/stocks/with-strategies`);
  }

  getHistorical(
    symbol: string,
    timeframe?: string,
    startDate?: string,
    endDate?: string,
  ): Observable<HistoricalResponse> {
    let params = new HttpParams();
    if (timeframe) params = params.set('timeframe', timeframe);
    if (startDate) params = params.set('start_date', startDate);
    if (endDate) params = params.set('end_date', endDate);
    return this.http.get<HistoricalResponse>(
      `${this.api}/stocks/historical/${symbol}`,
      { params },
    );
  }

  getIndicators(
    symbol: string,
    timeframe?: string,
  ): Observable<IndicatorsResponse> {
    let params = new HttpParams();
    if (timeframe) params = params.set('timeframe', timeframe);
    return this.http.get<IndicatorsResponse>(
      `${this.api}/indicators/${symbol}`,
      { params },
    );
  }
}
