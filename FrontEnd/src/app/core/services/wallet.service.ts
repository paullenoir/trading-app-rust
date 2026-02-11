import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';
import {
  WalletBalance,
  TransactionRequest,
  TransactionResponse,
  WalletHistoryItem,
} from '../models/wallet.model';

@Injectable({ providedIn: 'root' })
export class WalletService {
  private readonly api = environment.apiBaseUrl;

  constructor(private readonly http: HttpClient) {}

  getBalance(): Observable<WalletBalance> {
    return this.http.get<WalletBalance>(`${this.api}/wallet/balance`);
  }

  getHistory(): Observable<WalletHistoryItem[]> {
    return this.http.get<WalletHistoryItem[]>(`${this.api}/wallet/history`);
  }

  deposit(req: TransactionRequest): Observable<TransactionResponse> {
    return this.http.post<TransactionResponse>(`${this.api}/wallet/deposit`, req);
  }

  withdraw(req: TransactionRequest): Observable<TransactionResponse> {
    return this.http.post<TransactionResponse>(`${this.api}/wallet/withdraw`, req);
  }

  addTransaction(req: TransactionRequest): Observable<TransactionResponse> {
    return this.http.post<TransactionResponse>(`${this.api}/wallet/transaction`, req);
  }
}
