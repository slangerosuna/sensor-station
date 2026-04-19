import { Component, inject, input, OnInit, signal } from '@angular/core';
import { Station } from '../../../station-service';
import { HttpClient } from '@angular/common/http';
import { ButtonModule } from 'primeng/button';
@Component({
  selector: 'app-station-display-thermal-imaging',
  imports: [ButtonModule],
  templateUrl: './station-display-thermal-imaging.html',
  styleUrl: './station-display-thermal-imaging.css',
})
export class StationDisplayThermalImaging implements OnInit {
  _http = inject(HttpClient);
  
  station = input.required<Station>();
  pixels = signal<ThermalImagePixel[]>([]);
  
  water_ground_selected = signal<"water"|"ground"|"none">("none");
  ngOnInit() {
    this.get_thermal_image();
  }
  get_thermal_image(){
    const pixels: ThermalImagePixel[] = [];
    for(let x = 0; x < 24; x+=1){
        for(let y = 0; y < 32; y+=1){
          pixels.push({
            x: x,
            y: y,
            ground: false,
            water: false,
            temperature: 30,
          });
        }
      }
      this.pixels.set(pixels);
    };
  
  /*get_thermal_image() {
    const pixels: ThermalImagePixel[] = [];
    this._http.get<number[]>("/get_most_recent_image").subscribe((data) => {
      for(let x = 0; x < 24; x+=1){
        for(let y = 0; y < 32; y+=1){
          pixels.push({
            x: x,
            y: y,
            ground: false,
            water: false,
            temperature: data[x * 32 + y],
          });
        }
      }
    }); 
    this._http.request<string>("GET","/station/bitmap",{body:{surface: "water"}}).subscribe((data) => {
      for(let x = 0; x < 24; x+=1){
        for(let y = 0; y < 32; y+=1){
          pixels[x * 32 + y].water = data[x * 32 + y] === "1";
        }
      }
    });
    this._http.request<string>("GET","/station/bitmap",{body:{surface: "ground"}}).subscribe((data) => {
      for(let x = 0; x < 24; x+=1){
        for(let y = 0; y < 32; y+=1){
          pixels[x * 32 + y].ground = data[x * 32 + y] === "1";
        }
      }
    });
    this.pixels.set(pixels);

  }*/
  
  get_overlay_color(pixel: ThermalImagePixel): string {
    if(pixel.water){
      return "#0000FF40";
    }
    else if (pixel.ground){
      return "#8B451340";
    }
    return "transparent";
  }
  selectPixel(pixel: ThermalImagePixel) {
    if(this.water_ground_selected() === "none"){
      pixel.ground = false;
      pixel.water = false;
    }
    else if (this.water_ground_selected() === "ground"){
      pixel.ground = true;
      pixel.water = false;
    }
    else if (this.water_ground_selected() === "water"){
      pixel.ground = false;
      pixel.water = true;
    }
  }
  convert_temperature_to_color(temperature: number): string {
    return 'white'; 
  }
}
export type ThermalImagePixel = {
  x: number;
  y: number;
  ground: boolean;
  water: boolean;
  temperature: number;
}
