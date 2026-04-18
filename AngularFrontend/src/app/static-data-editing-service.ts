import { computed, inject, Injectable, linkedSignal, signal } from '@angular/core';
import { Station, StationService } from './station-service';
import { HttpClient } from '@angular/common/http';
import { TreeValidationResult } from '@angular/forms/signals';
import { catchError, firstValueFrom, of } from 'rxjs';

@Injectable({
  providedIn: 'root',
})
export class StaticDataEditingService {
  readonly _http = inject(HttpClient);
  readonly _station_service = inject(StationService);
  editing_station = signal<Station | null>(null);
  editing_station_nonnull = linkedSignal<Station>(() => {
    if(this.editing_station()){
      return this.editing_station()!;
    }
    else{
      return {
        lastUpdated: new Date(),
        StaticStationData: {
          id: 0,
          name: "",
          latitude: 0,
          longitude: 0,
          water_speed: 0,
          soil_moisture: 0,
        }
      } as Station;
    }
  });
  async submit(): Promise<TreeValidationResult> {
    if(this.editing_station()){
      try {
        await firstValueFrom(this._http.post("/api/update_station_static_data", this.editing_station()?.StaticStationData, { responseType: 'text' }));
        const stations = this._station_service.stations();
        if (stations) {
          this._station_service.stations.set(stations.map(station => {
            if (station.StaticStationData.id === this.editing_station()?.StaticStationData.id) {
              return {
                ...station,
                StaticStationData: this.editing_station()!.StaticStationData,
              } as Station;
            }
            return station;
          }));
        }

        return undefined;
      } catch (error) {
        return { kind: "ServerError", message: "Failed to update station data." };
      }
    }
  }
}
