import { Component, inject, input, OnInit, signal } from '@angular/core';
import { Station } from '../../../station-service';
import { MultiSelectModule } from 'primeng/multiselect';
import { FormsModule } from '@angular/forms';
import { DatePickerModule } from 'primeng/datepicker';
import { ChartModule } from 'primeng/chart';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Button } from "primeng/button";
@Component({
  selector: 'app-station-display-data',
  imports: [MultiSelectModule, FormsModule, DatePickerModule, ChartModule, Button],
  templateUrl: './station-display-data.html',
  styleUrl: './station-display-data.css',
})
export class StationDisplayData implements OnInit {
  _http = inject(HttpClient);
  
  readonly station = input.required<Station>();
  readonly data_options = [
    { label: "Air Temperature", value: "air_temperature" },
    { label: "Air Humidity", value: "humidity" },
    { label: "Co2 concentration (ppm)", value: "co2" },
    { label: "Barometric Pressure", value: "pressure" },
    { label: "Ground Temperature", value: "ground_temperature" },
    { label: "Water Temperature", value: "water_temperature" },
    { label: "Evaporation Rate", value: "rate_of_evaporation" },
    { label: "Net Irradiance", value: "net_irradiance" },
    { label: "Wind Speed", value: "wind_speed" },
  ]
  readonly current_date = new Date();
  selected_data = signal<{ label: string, value: string }[]>([]);
  start_datetime = signal<Date | null>(null);
  end_datetime = signal<Date | null>(new Date());
  data_loaded = signal<boolean>(false);
  chart_data: any;
  chart_options: any;
  ngOnInit(){

  }
  init_chart(){
    let params = new HttpParams()
      .set('preferred_rows', '200')
      .set('columns', this.selected_data().map(d => d.value).join(','));

    if (this.start_datetime()) {
      params = params.set('start', this.start_datetime()!.toISOString());
    }

    if (this.end_datetime()) {
      params = params.set('end', this.end_datetime()!.toISOString());
    }

    this._http.get<DataReturnValues[]>("api/recordings",{
      params,
    }
    ).subscribe((data) => {
      this.chart_data = {
        labels: data.map(d => d.timestamp.toString()),
        datasets: this.selected_data().map((d, index) => ({
          label: d.label,
          data: data.map(row => row[d.value as keyof DataReturnValues]),
        }))
      };
      this.data_loaded.set(true);
    });
  }
}
export type DataReturnValues = {
  timestamp: Date;
  air_temperature?: number;
  humidity?: number;
  co2?: number;
  pressure?: number;
  ground_temperature?: number;
  water_temperature?: number;
  rate_of_evaporation?: number;
  net_irradiance?: number;
  wind_speed?: number;
}