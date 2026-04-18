import { Injectable, signal } from '@angular/core';
import { Station } from './station-service';

@Injectable({
  providedIn: 'root',
})
export class StaticDataEditingService {
  editing_station = signal<Station | null>(null);
  
}
