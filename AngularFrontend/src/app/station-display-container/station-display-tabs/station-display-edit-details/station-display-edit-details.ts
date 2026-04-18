import { Component, inject, input, OnInit, signal } from '@angular/core';
import { StaticStationData, Station } from '../../../station-service';
import { form, FormField, max, required, min, FormRoot, submit } from '@angular/forms/signals';
import { HttpClient } from '@angular/common/http';
import { catchError } from 'rxjs';

@Component({
  selector: 'app-station-display-edit-details',
  imports: [FormField, FormRoot],
  templateUrl: './station-display-edit-details.html',
  styleUrl: './station-display-edit-details.css',
})
export class StationDisplayEditDetails implements OnInit {
  readonly _http = inject(HttpClient);

  station = input.required<Station>();
  readonly static_data_modal = signal<StaticStationData>({
    id: 0,
    name: "",
    latitude: 0,
    longitude: 0,
    water_speed: 0,
    soil_moisture: 0,
  });

  readonly static_data_form = form(this.static_data_modal, (path) => {
    required(path.name, { message: "Name is required" });
    required(path.latitude, { message: "Latitude is required" });
    min(path.latitude, -90, { message: "Latitude must be between -90 and 90" });
    max(path.latitude, 90, { message: "Latitude must be between -90 and 90" });
    required(path.longitude, { message: "Longitude is required" });
    min(path.longitude, -180, { message: "Longitude must be between -180 and 180" });
    max(path.longitude, 180, { message: "Longitude must be between -180 and 180" });
  }, { submission: { 
    action: async (form) => {
        try{await this._http.post(`example.com/api/stations/${this.station().StaticStationData.id}/update_static_data`, form().value(),{timeout: 5000}).toPromise();
        return undefined;
        } catch (error) {
        console.error("Failed to update static station data", error);
        return { kind: 'serverError', message: "Error updating static station data"} };
    }}});
  ngOnInit(): void {
    this.static_data_modal.set(this.station().StaticStationData);
  }
}
