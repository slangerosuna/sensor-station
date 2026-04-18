import { Component, inject, input, OnInit, signal } from '@angular/core';
import { StaticStationData, Station } from '../../../station-service';
import { form, FormField, max, required, min, FormRoot, submit } from '@angular/forms/signals';
import { HttpClient } from '@angular/common/http';
import { catchError } from 'rxjs';
import { StaticDataEditingService } from '../../../static-data-editing-service';

@Component({
  selector: 'app-station-display-edit-details',
  imports: [FormField],
  templateUrl: './station-display-edit-details.html',
  styleUrl: './station-display-edit-details.css',
})
export class StationDisplayEditDetails implements OnInit {
  readonly _static_data_editing_service = inject(StaticDataEditingService);
  station = input.required<Station>();
  

  readonly static_data_form = form(this._static_data_editing_service.editing_station_nonnull, (path) => {
    required(path.StaticStationData.name, { message: "Name is required" });
    required(path.StaticStationData.latitude, { message: "Latitude is required" });
    min(path.StaticStationData.latitude, -90, { message: "Latitude must be between -90 and 90" });
    max(path.StaticStationData.latitude, 90, { message: "Latitude must be between -90 and 90" });
    required(path.StaticStationData.longitude, { message: "Longitude is required" });
    min(path.StaticStationData.longitude, -180, { message: "Longitude must be between -180 and 180" });
    max(path.StaticStationData.longitude, 180, { message: "Longitude must be between -180 and 180" });
  });
  onSubmit(event: Event){
    event.preventDefault();
    submit(this.static_data_form, () => {return this._static_data_editing_service.submit();});
  }
  ngOnInit(): void {
    this._static_data_editing_service.editing_station.set(this.station());
  }
}
