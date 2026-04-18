import { ComponentFixture, TestBed } from '@angular/core/testing';

import { StationDisplayDetails } from './station-display-details';

describe('StationDisplayDetails', () => {
  let component: StationDisplayDetails;
  let fixture: ComponentFixture<StationDisplayDetails>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [StationDisplayDetails],
    }).compileComponents();

    fixture = TestBed.createComponent(StationDisplayDetails);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
