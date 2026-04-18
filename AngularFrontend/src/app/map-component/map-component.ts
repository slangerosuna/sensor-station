import { Component, OnInit } from '@angular/core';
import { LeafletDirective, LeafletLayersControlDirective, LeafletBaseLayersDirective } from '@bluehalo/ngx-leaflet';
import * as L from "leaflet";
import { Control } from 'leaflet';
@Component({
  selector: 'app-map-component',
  imports: [LeafletDirective, LeafletBaseLayersDirective, LeafletLayersControlDirective],
  templateUrl: './map-component.html',
  styleUrl: './map-component.css',
})
export class MapComponent implements OnInit {
  readonly osm_layer =L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {attribution: '&amp;copy; OpenStreetMap contributors'});
  readonly esri_satellite_layer = L.tileLayer('https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}',{attribution:'Tiles &copy; Esri &mdash; Source: Esri, i-cubed, USDA, USGS, AEX, GeoEye, Getmapping, Aerogrid, IGN, IGP, UPR-EGP, and the GIS User Community'})
  readonly baseLayers = {
      'OpenStreetMaps': this.osm_layer,
      'Satellite Imagery': this.esri_satellite_layer
  };
  readonly layer_control_options: Control.LayersOptions = {

  }
  options = {
    layers: [
      this.esri_satellite_layer
    ],
    zoom: 13,
    center: L.latLng([ 37.3597657, -120.4267336])
  }; 
  onMapReady(map: L.Map) {
    
  }
  ngOnInit(): void {
    
  }
}
