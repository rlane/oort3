#ifndef OORT_H
#define OORT_H

#include <cstdint>
#include <cstring>
#include <cstdlib>

extern "C" {
    extern uint64_t SYSTEM_STATE[128];
    extern uint8_t ENVIRONMENT[1024];
    extern uint8_t PANIC_BUFFER[1024];
}

enum SystemState {
    Class,
    Seed,
    PositionX,
    PositionY,
    VelocityX,
    VelocityY,
    Heading,
    AngularVelocity,

    AccelerateX,
    AccelerateY,
    Torque,

    Aim0,
    Aim1,
    Aim2,
    Aim3,

    Fire0,
    Fire1,
    Fire2,
    Fire3,

    Explode,

    RadarHeading,
    RadarWidth,
    RadarContactFound,
    RadarContactClass,
    RadarContactPositionX,
    RadarContactPositionY,
    RadarContactVelocityX,
    RadarContactVelocityY,

    DebugTextPointer,
    DebugTextLength,

    MaxForwardAcceleration,
    MaxLateralAcceleration,
    MaxAngularAcceleration,

    DebugLinesPointer,
    DebugLinesLength,

    RadarMinDistance,
    RadarMaxDistance,

    CurrentTick,
    MaxBackwardAcceleration,

    ActivateAbility,

    Radio0Channel, // TODO collapse into command word
    Radio0Send,
    Radio0Receive,
    Radio0Data0,
    Radio0Data1,
    Radio0Data2,
    Radio0Data3,

    Radio1Channel,
    Radio1Send,
    Radio1Receive,
    Radio1Data0,
    Radio1Data1,
    Radio1Data2,
    Radio1Data3,

    Radio2Channel,
    Radio2Send,
    Radio2Receive,
    Radio2Data0,
    Radio2Data1,
    Radio2Data2,
    Radio2Data3,

    Radio3Channel,
    Radio3Send,
    Radio3Receive,
    Radio3Data0,
    Radio3Data1,
    Radio3Data2,
    Radio3Data3,

    Radio4Channel,
    Radio4Send,
    Radio4Receive,
    Radio4Data0,
    Radio4Data1,
    Radio4Data2,
    Radio4Data3,

    Radio5Channel,
    Radio5Send,
    Radio5Receive,
    Radio5Data0,
    Radio5Data1,
    Radio5Data2,
    Radio5Data3,

    Radio6Channel,
    Radio6Send,
    Radio6Receive,
    Radio6Data0,
    Radio6Data1,
    Radio6Data2,
    Radio6Data3,

    Radio7Channel,
    Radio7Send,
    Radio7Receive,
    Radio7Data0,
    Radio7Data1,
    Radio7Data2,
    Radio7Data3,

    // TODO not part of interface
    SelectedRadio,

    DrawnTextPointer,
    DrawnTextLength,

    RadarEcmMode,

    Health,
    Fuel,

    RadarContactRssi,
    RadarContactSnr,

    ReloadTicks0,
    ReloadTicks1,
    ReloadTicks2,
    ReloadTicks3,

    Id,

    Size,
    MaxSize = 128,
};

inline uint64_t read_u64(enum SystemState key) {
    return SYSTEM_STATE[key];
}

inline double read_f64(enum SystemState key) {
    uint64_t u64_value = read_u64(key);
    double f64_value;
    std::memcpy(&f64_value, &u64_value, sizeof(f64_value));
    return f64_value;
}

inline void write_u64(enum SystemState key, uint64_t value) {
    SYSTEM_STATE[key] = value;
}

inline void write_f64(enum SystemState key, double value) {
    uint64_t u64_value;
    std::memcpy(&u64_value, &value, sizeof(u64_value));
    write_u64(key, u64_value);
}

#endif
