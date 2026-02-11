import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';
import {
  OpenTrade,
  OpenTradeWithRecommendations,
  ClosedTrade,
} from '../models/trade.model';

@Injectable({ providedIn: 'root' })
export class TradeService {
  private readonly api = environment.apiBaseUrl;

  constructor(private readonly http: HttpClient) {}

  getOpenTrades(): Observable<OpenTrade[]> {
    return this.http.get<OpenTrade[]>(`${this.api}/trades/open`);
  }

  getOpenTradesWithRecommendations(): Observable<OpenTradeWithRecommendations[]> {
    return this.http.get<OpenTradeWithRecommendations[]>(
      `${this.api}/trades/open-with-recommendations`,
    );
  }

  getClosedTrades(): Observable<ClosedTrade[]> {
    return this.http.get<ClosedTrade[]>(`${this.api}/trades/closed`);
  }

  getAllTrades(): Observable<unknown[]> {
    return this.http.get<unknown[]>(`${this.api}/trades`);
  }
}
