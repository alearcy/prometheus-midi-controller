import React, {memo, useState} from 'react';
import {useSelector, useDispatch} from 'react-redux';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import classNames from 'classnames';
import { ElementTypes, MidiTypes } from '../constants/genericConstants';
import OscButton from './OscButton';
import Label from './Label';
import Nouislider from "nouislider-react";
import "nouislider/distribute/nouislider.css";
import {sendMIDIMessage} from "../store/actions";

const Element = ({
    obj,
    isEditingMode,
    editElement,
    toggleStatic,
    socket
}) => {

    const currentTab = useSelector(state => state.tabs.currentTab);
    const [currentValue, setCurrentValue] = useState(0)

    let dispatch = useDispatch();

    const handleResetPitch = (obj) => {
        if (obj.midiType === MidiTypes.PITCH) {
            setCurrentValue(0);
        }
    }

    const pitchBendRangeMap = (v) => {
        var minM = -1.0;
        var maxM = 1.0;
        var minV = 0;
        var maxV = 16383;
        var scal = (maxV - minV) / (maxM - minM);
        return Math.trunc(minV + scal * (v - minM)); 
    };

    const sendBtnMsg = (obj) => {
        sendMidiFromButtons(obj);
    }

    const sendMidiFromButtons = (obj) => {
        const value = obj.midiType === MidiTypes.NOTE ? obj.value : 127;
        const data = {
            midi_type: obj.midiType,
            channel: obj.channel,
            cc_value: 0,
            value: value,
        };
        const message = JSON.stringify(data);
        socket.send(message);
        sendFormattedMidiButtonMessage(data);
    }

    const sendFormattedMidiButtonMessage = (data) => {
        const msg = `Type: ${data.midi_type}, Channel: ${data.channel}, value: ${data.value}`;
        dispatch(sendMIDIMessage(msg));
    }

    const sendSlideValue = (obj, v) => {
        const value = Array.isArray(v) ? v[0] : 0
        setCurrentValue(value);
        sendMidiFromSliders(obj, value);
    }

    const sendMidiFromSliders = (obj, v) => {
        const data = {
            midi_type: obj.midiType,
            channel: obj.channel,
            cc_value: obj.ccValue,
            value:
                obj.midiType === MidiTypes.PITCH
                ? pitchBendRangeMap(parseFloat(v))
                : Math.floor(v),
        };
        const message = JSON.stringify(data);
        socket.send(message);
        sendFormattedMidiSliderMessage(data);
    }

    const sendFormattedMidiSliderMessage = (data) => {
        let msg = '';
        if (data.midi_type === MidiTypes.CC || data.type === MidiTypes.NOTE) {
            msg = `Type: ${data.midi_type}, Channel: ${data.channel}, value1: ${data.cc_value}, value2: ${data.value}`;
        } else {
            msg = `Type: ${data.midi_type}, Channel: ${data.channel}, value: ${data.value}`;
        }
        dispatch(sendMIDIMessage(msg))
    }

    const elementsMap = {
        [ElementTypes.BTN]: (
            <OscButton
            obj={obj}
            onPointerDown={isEditingMode ? null : () => sendBtnMsg(obj)}
            className="button-wrapper"
            />
        ),
        [ElementTypes.SLIDER]: (
            <div>
            <Nouislider
                key={obj.id}
                id={obj.id}
                connect
                animate={false}
                start={currentValue}
                behaviour="drag"
                range={{
                min: [
                    obj.midiType === MidiTypes.CC
                    ? obj.minCcValue
                    : obj.minPitchValue,
                ],
                max: [
                    obj.midiType === MidiTypes.CC
                    ? obj.maxCcValue
                    : obj.maxPitchValue,
                ],
                }}
                direction="rtl"
                step={obj.midiType === MidiTypes.CC ? 1 : 0.001}
                orientation={obj.orientation}
                disabled={isEditingMode}
                onSlide={(v) => sendSlideValue(obj, v)}
                onEnd={() => handleResetPitch(obj)}
            />
            <div className="slider-label" style={{ color: obj.labelColor }}>
                {obj.label}
            </div>
            </div>
        ),
        [ElementTypes.LABEL]: <Label obj={obj} />,
    };
    const elementClass = classNames({
        oscButton: obj.type === ElementTypes.BTN,
        slider: obj.type === ElementTypes.SLIDER,
        label: obj.type === ElementTypes.LABEL,
        editingMode: isEditingMode,
    });
    const elementStyles = {
        backgroundColor: obj.type === ElementTypes.SLIDER ? null : obj.styleColor,
        borderColor: obj.type === ElementTypes.SLIDER ? obj.styleColor : null,
        color: obj.labelColor,
    };
    return (
        <div className={elementClass} style={elementStyles}>
            {elementsMap[obj.type]}
            {isEditingMode ? (
                <div className="padlock" onPointerDown={() => toggleStatic(obj)}>
                    {obj.static ? (
                        <FontAwesomeIcon icon="lock" />
                    ) : (
                            <FontAwesomeIcon icon="lock-open" />
                        )
                    }
                </div>
            ) : null}
            {isEditingMode && !obj.static ? (
                <div className="edit" onPointerDown={() => editElement(obj.id)}>
                    <FontAwesomeIcon icon="pen" />
                </div>
            ) : null}
            {isEditingMode && !obj.static ? (
                <div className="draggable">
                    <FontAwesomeIcon icon="ellipsis-h" />
                </div>
            ) : null}
            {isEditingMode && !obj.static ? (
                <div className="resizable">
                    <FontAwesomeIcon icon="expand" />
                </div>
            ) : null}
        </div>
    );
};

export default memo(Element);
