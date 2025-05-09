package co.realfit.example;

import android.app.NativeActivity;
import android.content.ClipData;
import android.content.ClipboardManager;
import android.content.Context;
import android.media.midi.MidiDeviceInfo;
import android.os.Bundle;
import android.os.Handler;
import android.util.Log;
import android.view.inputmethod.InputMethodManager;
import android.media.midi.MidiManager;
import java.util.ArrayList;

/**
 * Application MainActivity handles UI and Midi device hotplug event from
 * native side.
 */
public class MainActivity extends NativeActivity {
    
    private static final String TAG = MainActivity.class.getName();;

    private AppMidiManager mAppMidiManager;

    // Connected devices
    private ArrayList<MidiDeviceInfo> mReceiveDevices = new ArrayList<MidiDeviceInfo>();
    private ArrayList<MidiDeviceInfo> mSendDevices = new ArrayList<MidiDeviceInfo>();
    private ArrayList<MidiDeviceInfo> midiDevices = new ArrayList<MidiDeviceInfo>();

    // Send Widgets
    // Spinner mOutputDevicesSpinner;

    // SeekBar mControllerSB;
    // SeekBar mPitchBendSB;

    // EditText mProgNumberEdit;

    // Receive Widgets
    // Spinner mInputDevicesSpinner;
    // TextView mReceiveMessageTx;

    // Force to load the native library
    static {
        AppMidiManager.loadNativeAPI();
        AppMidiManager.loadSynthAPI();
    }

    // static {
    //     System.loadLibrary("example");
    // }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        //
        // Init JNI for data receive callback
        //
        initNative();

        //
        // Setup the MIDI interface
        //
        MidiManager midiManager = (MidiManager) getSystemService(Context.MIDI_SERVICE);
        midiManager.registerDeviceCallback(new MidiDeviceCallback(), new Handler());
        
        mAppMidiManager = new AppMidiManager(midiManager);

        ScanMidiDevices();
    }

    private void showKeyboard() {
        Log.d("MainActivity", "showKeyboard instance method called");
        InputMethodManager inputManager = getSystemService(InputMethodManager.class);
        inputManager.showSoftInput(getWindow().getDecorView(), InputMethodManager.SHOW_IMPLICIT);
    }

    private void hideKeyboard() {
        Log.d("MainActivity", "hideKeyboard instance method called");
        InputMethodManager inputManager = getSystemService(InputMethodManager.class);
        inputManager.hideSoftInputFromWindow(getWindow().getDecorView().getWindowToken(), 0);
    }

    private String readClipboard() {
        ClipboardManager clipboardManager = (ClipboardManager) getApplicationContext().getSystemService(Context.CLIPBOARD_SERVICE);
        ClipData data = clipboardManager.getPrimaryClip();
        if (data == null) {
            Log.d("MainActivity", "ClipData in readClipboard is null");
            return "";
        }
        ClipData.Item item = data.getItemAt(0);
        if (item == null) {
        }
        return item.coerceToText(this).toString();
    }

    private void writeClipboard(String value) {
        ClipboardManager clipboardManager = (ClipboardManager) getApplicationContext().getSystemService(Context.CLIPBOARD_SERVICE);
        ClipData data = ClipData.newPlainText("MainActivity text", value);
        clipboardManager.setPrimaryClip(data);
    }

    /**
     * Device Scanning
     * Methods are called by the system whenever the set of attached devices changes.
     */
    private class MidiDeviceCallback extends MidiManager.DeviceCallback {
        @Override
        public void onDeviceAdded(MidiDeviceInfo device) {
            ScanMidiDevices();
        }

        @Override
        public void onDeviceRemoved(MidiDeviceInfo device) {
            ScanMidiDevices();
        }
    }

    /**
     * Scans and gathers the list of connected physical devices,
     * then calls onDeviceListChange() to update the UI. This has the
     * side-effect of causing a list item to be selected, which then
     * invokes the listener logic which connects the device(s).
     */
    private void ScanMidiDevices() {
        mAppMidiManager.ScanMidiDevices(mSendDevices, mReceiveDevices);
        onDeviceListChange();
    }

    /**
     * A class to hold MidiDevices in the list controls.
     */
    private class MidiDeviceListItem {
        private MidiDeviceInfo mDeviceInfo;

        public MidiDeviceListItem(MidiDeviceInfo deviceInfo) {
            mDeviceInfo = deviceInfo;
        }

        public MidiDeviceInfo getDeviceInfo() { return mDeviceInfo; }

        @Override
        public String toString() {
            return mDeviceInfo.getProperties().getString(MidiDeviceInfo.PROPERTY_NAME);
        }
    }

    /**
     * Fills the specified list control with a set of MidiDevices
     * @param spinner   The list control.
     * @param devices   The set of MidiDevices.
     */
    private void fillDeviceList(ArrayList<MidiDeviceInfo> devices) {
        midiDevices.clear();
        mAppMidiManager.clearKnownDevs();

        for(MidiDeviceInfo devInfo : devices) {
//            listItems.add(new MidiDeviceListItem(devInfo));
            // add to "known devices" list and send device name to rust.
            midiDevices.add(devInfo);
            mAppMidiManager.newMidiDev(devInfo.getProperties().getString(MidiDeviceInfo.PROPERTY_NAME));
        }
        // sendMidiMessagen
        // spinner.setAdapter(dataAdapter);
    }

    /**
     * Fills the Input & Output UI device list with the current set of MidiDevices for each type.
     */
    private void onDeviceListChange() {
        // fillDeviceList(mOutputDevicesSpinner, mReceiveDevices);
        // fillDeviceList(mInputDevicesSpinner, mSendDevices);
        fillDeviceList(mReceiveDevices);
        fillDeviceList(mSendDevices);
    }

    //
    // Native Interface methods
    //
    private native void initNative();

    /**
     * Called from the native code when MIDI messages are received.
     * @param message
     */
    private void onNativeMessageReceive(final byte[] message) {
        //
        // send midi messages to rust
        //

        mAppMidiManager.sendMidiMessage(message);
    }
}
