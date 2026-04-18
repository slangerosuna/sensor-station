import { TestBed } from '@angular/core/testing';

import { StaticDataEditingService } from './static-data-editing-service';

describe('StaticDataEditingService', () => {
  let service: StaticDataEditingService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(StaticDataEditingService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
