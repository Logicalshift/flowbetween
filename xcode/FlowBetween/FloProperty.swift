//
//  FloProperty.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 04/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation

///
/// Value of a ViewModel property
///
public enum PropertyValue {
    case Nothing;
    case Bool(Bool);
    case Int(Int64);
    case Float(Float64);
    case String(String);
}


///
/// Bridging class used to pass strings from the rust to the swift side of things
///
@objc(FloProperty) public class FloProperty : NSObject {
    var _value: PropertyValue;
    
    override init() {
        _value = PropertyValue.Nothing;
        
        super.init();
    }
    
    @objc(initWithBool:) init(withBool: Bool) {
        _value = PropertyValue.Bool(withBool);
        
        super.init();
    }
    
    @objc(initWithInt:) init(withInt: Int64) {
        _value = PropertyValue.Int(withInt);
        
        super.init();
    }
    
    @objc(initWithFloat:) init(withFloat: Float64) {
        _value = PropertyValue.Float(withFloat);
        
        super.init();
    }
    
    @objc(initWithString:) init(withString: NSString) {
        _value = PropertyValue.String(withString as String);
        NSLog("Property init with string: %@", withString);
        
        super.init();
    }
    
    @objc(initWithBinding:viewModel:) init(withBinding: UInt64, viewModel: FloViewModel) {
        _value = PropertyValue.String("Bindings not implemented");
        
        super.init();
    }

    var value: PropertyValue {
        get { return _value;}
    }
}
