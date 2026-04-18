import { ComponentFixture, TestBed } from '@angular/core/testing';

import { StationDisplayStaticDetails } from './station-display-static-details';

describe('StationDisplayStaticDetails', () => {
  let component: StationDisplayStaticDetails;
  let fixture: ComponentFixture<StationDisplayStaticDetails>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [StationDisplayStaticDetails],
    }).compileComponents();

    fixture = TestBed.createComponent(StationDisplayStaticDetails);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
