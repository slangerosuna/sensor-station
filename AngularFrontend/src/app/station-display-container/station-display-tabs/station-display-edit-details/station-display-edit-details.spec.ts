import { ComponentFixture, TestBed } from '@angular/core/testing';

import { StationDisplayEditDetails } from './station-display-edit-details';

describe('StationDisplayEditDetails', () => {
  let component: StationDisplayEditDetails;
  let fixture: ComponentFixture<StationDisplayEditDetails>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [StationDisplayEditDetails],
    }).compileComponents();

    fixture = TestBed.createComponent(StationDisplayEditDetails);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
