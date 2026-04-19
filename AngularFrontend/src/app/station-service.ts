import { HttpClient, httpResource } from '@angular/common/http';
import { inject, Injectable, signal, Signal } from '@angular/core';
import { exhaustMap, Observable, Subject, switchMap, tap } from 'rxjs';

@Injectable({
  providedIn: 'root',
})
export class StationService {
  server_url: Signal<string> = signal('http://localhost:8080');
  readonly stationSubject$ = new Subject();
  private readonly _http = inject(HttpClient);

  /*readonly stations = httpResource<Station[]|null>(() => ({
    url: `${this.server_url()}/stations`,
    method: 'GET',
    initialValue: null,
    reloadOn: this.stationSubject$
  }));*/

  stations = signal<Station[]|null>([{lastUpdated: new Date(new Date().getTime() - 20 * 60 * 1000), StaticStationData: {    
    id: 1,
    name: "Station 1",
    latitude: 37.3597657,
    longitude: -120.4267336,
    water_depth: 0,
    soil_moisture: 0
  }} as Station]);

  reload_stations() {
    this.stationSubject$.next(null);
  }

}
export interface Station {
  lastUpdated: Date;
  StaticStationData: StaticStationData;
}
export type StaticStationData = {
  id: number;
  name: string;
  latitude: number;
  longitude: number;
  water_depth: number;
  soil_moisture: number;
}

