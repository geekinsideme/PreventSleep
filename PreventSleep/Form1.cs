using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using System.Timers;
using System.Windows.Forms;
using Microsoft.VisualBasic.FileIO;
using System.Drawing.Imaging;
using static System.Net.Mime.MediaTypeNames;
using Timer = System.Timers.Timer;
using Svg;

namespace PreventSleep
{

    public partial class Form1 : Form
    {
        private MenuStrip menuStrip;
        private ToolStripMenuItem dummyMenuItem;
        private ToolStripMenuItem shortcutMenuItem;
        private int numOfScreen = 1;
        private FormWindowState lastWindowState = FormWindowState.Normal;
        private static Timer topMostTimer;
        private bool topMostTimerUp = false;

        [DllImport("user32.dll", CharSet = CharSet.Auto)]
        private static extern IntPtr SendMessage(IntPtr hWnd, uint Msg, IntPtr wParam, IntPtr lParam);
        private static readonly IntPtr HWND_BROADCAST = new IntPtr(0xffff);
        private const uint WM_SYSCOMMAND = 0x0112;
        private const int SC_MONITORPOWER = 0xf170;
        private const int MonitorShutoff = 2;

        private static Guid GUID_MONITOR_POWER_ON = new Guid("02731015-4510-4526-99e6-e5a17ebd1aea");
        private const int WM_POWERBROADCAST = 0x0218;
        private const int DEVICE_NOTIFY_WINDOW_HANDLE = 0x00000000;
        private const int PBT_POWERSETTINGCHANGE = 0x8013;

        [DllImport("User32", SetLastError = true, EntryPoint = "RegisterPowerSettingNotification", CallingConvention = CallingConvention.StdCall)]
        private static extern IntPtr RegisterPowerSettingNotification(IntPtr hRecipient, ref Guid PowerSettingGuid, Int32 Flags);

        [StructLayout(LayoutKind.Sequential, Pack = 4)]
        internal struct POWERBROADCAST_SETTING
        {
            public Guid PowerSetting;
            public uint DataLength;
            public byte Data;
        }

        public Form1(string[] args)
        {
            if (args.Length > 0 && args[0] == "set")
            {
                btnLocationSet_Click(null, null);
                Environment.Exit(0);
            }
            else if (args.Length > 0 && args[0] == "monitoroff")
            {
                SendMessage(HWND_BROADCAST, WM_SYSCOMMAND, (IntPtr)SC_MONITORPOWER, (IntPtr)MonitorShutoff);
                Environment.Exit(0);
            }
            else
            {
                InitializeComponent();
                if (args.Length > 0 && args[0] == "noprevent")
                {
                    preventSleep.Checked = false;
                    SetThreadExecutionState(ExecutionState.Continuous);
                    timer1.Enabled = preventSleep.Checked;
                }
            }

            IntPtr hWnd = this.Handle;
            IntPtr ret = RegisterPowerSettingNotification(hWnd, ref GUID_MONITOR_POWER_ON, DEVICE_NOTIFY_WINDOW_HANDLE);
            Debug.WriteLine("Registered: " + ret.ToString());
            Debug.WriteLine("LastError:" + Marshal.GetLastWin32Error().ToString());

            menuStrip = new MenuStrip();
            dummyMenuItem = new ToolStripMenuItem();
            shortcutMenuItem = new ToolStripMenuItem();

            // メニューアイテムを追加（これは表示されません）
            dummyMenuItem.DropDownItems.Add(shortcutMenuItem);
            menuStrip.Items.Add(dummyMenuItem);

            // メニューストリップをフォームに追加（これも表示されません）
            this.Controls.Add(menuStrip);
            this.MainMenuStrip = menuStrip;
            menuStrip.Visible = false;
        }

        private void shortcutMenuItem_Click(object sender, EventArgs e)
        {
            btnLocationSet_Click(null, null);
        }

        private async void MonitorPowerStatusChanged()
        {
            await Task.Delay(2000); // 2秒待機
            btnLocationSet_Click(null, null);
        }

        protected override void WndProc(ref Message m)
        {
            if (m.Msg == WM_POWERBROADCAST)
            {
                if (m.WParam.ToInt32() == PBT_POWERSETTINGCHANGE)
                {
                    POWERBROADCAST_SETTING pps = (POWERBROADCAST_SETTING)Marshal.PtrToStructure(m.LParam, typeof(POWERBROADCAST_SETTING));
                    if (pps.PowerSetting == GUID_MONITOR_POWER_ON)
                    {
                        Debug.WriteLine("Monitor power status changed: " + pps.Data);
                        MonitorPowerStatusChanged();
                    }
                }
            }
            base.WndProc(ref m);
        }

        private void Form1_Load(object sender, EventArgs e)
        {
            read_settings();
            if (preventSleep.Checked)
            {
                //画面暗転阻止
                SetThreadExecutionState(ExecutionState.DisplayRequired | ExecutionState.Continuous);
                timer1.Enabled = true;
            }
            timer2.Enabled = true;

            topMostTimer = new Timer();
            topMostTimer.Elapsed += OnTopMostTimerEvnet;

            this.Left = Screen.PrimaryScreen.WorkingArea.Left;
            this.Top = Screen.PrimaryScreen.WorkingArea.Top + Screen.PrimaryScreen.WorkingArea.Height - this.Height;
        }

        private void read_settings()
        {
            // 設定読み込み
            TextFieldParser parser = new TextFieldParser("PreventSleep.txt", Encoding.GetEncoding("UTF-8"));
            parser.TextFieldType = FieldType.Delimited;
            parser.SetDelimiters(",");
            // 座標セット
            setpos_list.Clear();
            while (parser.EndOfData == false)
            {
                string[] columns = parser.ReadFields();
                setpos_list.Add(columns);
            }
        }

        /// <summary>
        /// 画面暗転防止
        /// </summary>

        [FlagsAttribute]
        public enum ExecutionState : uint
        {
            // 関数が失敗した時の戻り値
            Null = 0,
            // スタンバイを抑止
            SystemRequired = 1,
            // 画面OFFを抑止
            DisplayRequired = 2,
            // 効果を永続させる。ほかオプションと併用する。
            Continuous = 0x80000000,
        }
        [DllImport("kernel32.dll")]
        extern static ExecutionState SetThreadExecutionState(ExecutionState esFlags);
        [DllImport("user32.dll")]
        extern static uint SendInput(
            uint nInputs,   // INPUT 構造体の数(イベント数)
            INPUT[] pInputs,   // INPUT 構造体
            int cbSize     // INPUT 構造体のサイズ
        );
        [StructLayout(LayoutKind.Sequential)]
        struct INPUT
        {
            public int type;  // 0 = INPUT_MOUSE(デフォルト), 1 = INPUT_KEYBOARD
            public MOUSEINPUT mi;
        }
        [StructLayout(LayoutKind.Sequential)]
        struct MOUSEINPUT
        {
            public int dx;
            public int dy;
            public int mouseData;  // amount of wheel movement
            public int dwFlags;
            public int time;  // time stamp for the event
            public IntPtr dwExtraInfo;
        }
        // dwFlags
        const int MOUSEEVENTF_MOVED = 0x0001;
        const int MOUSEEVENTF_LEFTDOWN = 0x0002;  // 左ボタン Down
        const int MOUSEEVENTF_LEFTUP = 0x0004;  // 左ボタン Up
        const int MOUSEEVENTF_RIGHTDOWN = 0x0008;  // 右ボタン Down
        const int MOUSEEVENTF_RIGHTUP = 0x0010;  // 右ボタン Up
        const int MOUSEEVENTF_MIDDLEDOWN = 0x0020;  // 中ボタン Down
        const int MOUSEEVENTF_MIDDLEUP = 0x0040;  // 中ボタン Up
        const int MOUSEEVENTF_WHEEL = 0x0080;
        const int MOUSEEVENTF_XDOWN = 0x0100;
        const int MOUSEEVENTF_XUP = 0x0200;
        const int MOUSEEVENTF_ABSOLUTE = 0x8000;
        const int screen_length = 0x10000;  // for MOUSEEVENTF_ABSOLUTE (この値は固定)

        private void timer1_Tick(object sender, EventArgs e)
        {
            if (preventSleep.Checked)
            {
                //画面暗転阻止
                SetThreadExecutionState(ExecutionState.DisplayRequired | ExecutionState.Continuous);
                // ドラッグ操作の準備 (struct 配列の宣言)
                INPUT[] input = new INPUT[1];  // イベントを格納
                                               // ドラッグ操作の準備 (イベントの定義 = 相対座標へ移動)
                input[0].mi.dx = 0;  // 相対座標で0　つまり動かさない
                input[0].mi.dy = 0;  // 相対座標で0 つまり動かさない
                input[0].mi.dwFlags = MOUSEEVENTF_MOVED;
                // ドラッグ操作の実行 (イベントの生成)
                SendInput(1, input, Marshal.SizeOf(input[0]));
            }
        }

        string path = File.ReadAllText("PreventSleep.txt");
        DateTime timestamp = DateTime.Now;
        private void timer2_Tick(object sender, EventArgs e)
        {
            try
            {
                // 最小化から復帰したときには、前面に表示
                if (lastWindowState == FormWindowState.Minimized && this.WindowState != FormWindowState.Minimized)
                {
                    this.TopMost = true;
                    topMostTimerUp = false;
                    topMostTimer.AutoReset = false;
                    topMostTimer.Interval = 5000;
                    topMostTimer.Enabled = true;
                }
                lastWindowState = this.WindowState;
                if (topMostTimerUp)
                {
                    this.TopMost = false;
                    topMostTimerUp = false;
                }

                //　位置補正
                bool isOnScreen = false;
                foreach (System.Windows.Forms.Screen screen in System.Windows.Forms.Screen.AllScreens)
                {
                    if (screen.WorkingArea.Left <= this.Left && this.Left <= (screen.WorkingArea.Left + screen.WorkingArea.Width) && screen.WorkingArea.Top <= this.Top && this.Top <= (screen.WorkingArea.Top + screen.WorkingArea.Height))
                    {
                        isOnScreen = true;
                        break;
                    }
                }
                if (!isOnScreen && this.WindowState != FormWindowState.Minimized)
                {
                    this.Left = Screen.PrimaryScreen.WorkingArea.Left;
                    this.Top = Screen.PrimaryScreen.WorkingArea.Top + Screen.PrimaryScreen.WorkingArea.Height - this.Height;
                }

                // スクリーン数追従
                if (System.Windows.Forms.Screen.AllScreens.Length != numOfScreen)
                {
                    numOfScreen = System.Windows.Forms.Screen.AllScreens.Length;
                    btnLocationSet_Click(null, null);
                }
            }
            catch (Exception)
            {
                // エラーは無視
            }
        }

        private void OnTopMostTimerEvnet(Object source, ElapsedEventArgs e)
        {
            topMostTimerUp = true;
        }

        private void checkBox1_CheckedChanged(object sender, EventArgs e)
        {

        }

        private void preventSleep_CheckedChanged(object sender, EventArgs e)
        {
            //画面暗転阻止
            if (preventSleep.Checked)
            {
                SetThreadExecutionState(ExecutionState.DisplayRequired | ExecutionState.Continuous);
            }
            else
            {
                SetThreadExecutionState(ExecutionState.Continuous);
            }
            timer1.Enabled = preventSleep.Checked;
        }

        private void btnListWindows_Click(object sender, EventArgs e)
        {
            windows_list = "";
            EnumWindows(new EnumWindowsDelegate(EnumWindowCallBack2), IntPtr.Zero);
            info.Text = windows_list;
        }

        // ウィンドウ再配置用の構造体
        private struct WindowInfo
        {
            public IntPtr hWnd;
            public string windowText;
            public string className;
            public wRECT rect;
            public bool isSpecified;  // PreventSleep.txtで指定されているか
        }

        private List<WindowInfo> allWindows = new List<WindowInfo>();

        private bool EnumAllWindowsCallBack(IntPtr hWnd, IntPtr lparam)
        {
            // ウィンドウが表示されていているかチェック
            if (!IsWindowVisible(hWnd))
            {
                return true;
            }

            // ウィンドウの状態を確認（最小化・最大化を除外）
            WINDOWPLACEMENT placement = new WINDOWPLACEMENT();
            placement.length = Marshal.SizeOf(placement);
            GetWindowPlacement(hWnd, ref placement);

            // 最小化(2)・最大化(3)されているウィンドウは除外
            if (placement.showCmd == 2 || placement.showCmd == 3)
            {
                return true;
            }

            // ウィンドウのタイトルを取得
            int textLen = GetWindowTextLength(hWnd);
            string windowText = "";
            if (textLen > 0)
            {
                StringBuilder tsb = new StringBuilder(textLen + 1);
                GetWindowText(hWnd, tsb, tsb.Capacity);
                windowText = tsb.ToString();
            }

            // ウィンドウのクラス名を取得
            StringBuilder csb = new StringBuilder(256);
            GetClassName(hWnd, csb, csb.Capacity);
            string className = csb.ToString();

            // タイトルのないウィンドウは対象外
            if (string.IsNullOrWhiteSpace(windowText))
            {
                return true;
            }

            // デスクトップ/タスクバー/UWP内部系のシステムウィンドウは対象外
            if ((windowText == "Program Manager" && className == "Progman") || className == "Shell_TrayWnd" || className == "Shell_SecondaryTrayWnd" || className == "Windows.UI.Core.CoreWindow")
            {
                return true;
            }

            // ウィンドウの位置・サイズを取得
            wRECT rect;
            GetWindowRect(hWnd, out rect);

            WindowInfo info = new WindowInfo();
            info.hWnd = hWnd;
            info.windowText = windowText;
            info.className = className;
            info.rect = rect;
            info.isSpecified = false;

            allWindows.Add(info);
            return true;
        }

        /// <summary>
        /// ウィンドウが PreventSleep.txt の設定に一致するかを判定
        /// </summary>
        private bool IsWindowMatched(WindowInfo window, string[] setting, int numDisplay)
        {
            if (setting[0].StartsWith("####") || setting[0].StartsWith("#"))
            {
                return false;
            }

            // 画面数チェック
            string disp_num = "12345";
            if (setting.Length > 6)
            {
                disp_num = setting[6];
            }

            if (!disp_num.Contains(numDisplay.ToString()))
            {
                return false;
            }

            // ウィンドウテキストチェック
            if (setting[0] != null && setting[0] != "")
            {
                if (!Regex.IsMatch(window.windowText, setting[0]))
                {
                    return false;
                }
            }

            // クラス名チェック
            if (setting[1] != null && setting[1] != "")
            {
                if (!Regex.IsMatch(window.className, setting[1]))
                {
                    return false;
                }
            }

            return true;
        }

        /// <summary>
        /// 指定されたスクリーン内で、左上が収まるスクリーンを取得
        /// </summary>
        private System.Windows.Forms.Screen FindScreenForPosition(int left, int top)
        {
            foreach (System.Windows.Forms.Screen screen in System.Windows.Forms.Screen.AllScreens)
            {
                if (screen.WorkingArea.Left <= left && left < screen.WorkingArea.Right &&
                    screen.WorkingArea.Top <= top && top < screen.WorkingArea.Bottom)
                {
                    return screen;
                }
            }
            return null;
        }

        private void location_set(int numDisplay)
        {
            read_settings();
            allWindows.Clear();
            StringBuilder relocationLog = new StringBuilder();

            // 各画面の作業領域（タスクバーを除く）を列挙
            System.Windows.Forms.Screen[] allScreens = System.Windows.Forms.Screen.AllScreens;
            for (int i = 0; i < allScreens.Length; i++)
            {
                System.Drawing.Rectangle wa = allScreens[i].WorkingArea;
                relocationLog.AppendFormat("# {0}, {1}, {2}, {3}, {4}\r\n", i + 1, wa.Left, wa.Top, wa.Width, wa.Height);
            }
            relocationLog.AppendLine();

            // すべてのウィンドウを列挙
            EnumWindows(new EnumWindowsDelegate(EnumAllWindowsCallBack), IntPtr.Zero);

            int shiftX = 0, shiftY = 0;  // シフト配置用の累積値
            int titleHeight = 25;  // ウィンドウタイトル高さ（概算）

            // 各ウィンドウを処理
            foreach (WindowInfo window in allWindows)
            {
                int oldLeft = window.rect.left;
                int oldTop = window.rect.top;
                int oldWidth = window.rect.right - window.rect.left;
                int oldHeight = window.rect.bottom - window.rect.top;

                int left = window.rect.left;
                int top = window.rect.top;
                int width = window.rect.right - window.rect.left;
                int height = window.rect.bottom - window.rect.top;
                bool isSpecified = false;

                // PreventSleep.txtの設定をチェック
                foreach (string[] setting in setpos_list)
                {
                    if (IsWindowMatched(window, setting, numDisplay))
                    {
                        // マッチしたので、その位置を使用
                        left = Convert.ToInt32(setting[2]);
                        top = Convert.ToInt32(setting[3]);
                        width = Convert.ToInt32(setting[4]);
                        height = Convert.ToInt32(setting[5]);
                        isSpecified = true;
                        break;
                    }
                }

                // マッチしなかった場合は、現在の表示位置を使用
                if (!isSpecified)
                {
                    left = window.rect.left;
                    top = window.rect.top;
                    width = window.rect.right - window.rect.left;
                    height = window.rect.bottom - window.rect.top;
                }

                if (width < 1)
                {
                    width = 1;
                }
                if (height < 1)
                {
                    height = 1;
                }

                // 左上が画面内かを確認
                System.Windows.Forms.Screen targetScreen = FindScreenForPosition(left, top);

                if (targetScreen == null)
                {
                    // 画面外の場合は、シフト配置した位置を使用
                    targetScreen = System.Windows.Forms.Screen.PrimaryScreen;
                    left = targetScreen.WorkingArea.Left + shiftX;
                    top = targetScreen.WorkingArea.Top + targetScreen.WorkingArea.Height - titleHeight - shiftY;

                    // シフト値を更新（次のウィンドウ用）
                    shiftX += titleHeight;
                    shiftY += titleHeight;

                    // 誤ったシフト値のリセット
                    if (shiftX > targetScreen.WorkingArea.Width / 2)
                    {
                        shiftX = 0;
                        shiftY = 0;
                    }
                }
                else
                {
                    // 画面内の場合はシフト値をリセット
                    shiftX = 0;
                    shiftY = 0;
                }

                // 右下が画面内に収まらない場合は、幅・高さは維持したまま左上を補正
                if (left + width > targetScreen.WorkingArea.Right)
                {
                    left = targetScreen.WorkingArea.Right - width;
                }
                if (top + height > targetScreen.WorkingArea.Bottom)
                {
                    top = targetScreen.WorkingArea.Bottom - height;
                }

                // 右下補正の結果、左上が画面外に出た場合は WorkingArea の左上にクランプしてサイズを縮小
                if (left < targetScreen.WorkingArea.Left)
                {
                    width -= targetScreen.WorkingArea.Left - left;
                    left = targetScreen.WorkingArea.Left;
                }
                if (top < targetScreen.WorkingArea.Top)
                {
                    height -= targetScreen.WorkingArea.Top - top;
                    top = targetScreen.WorkingArea.Top;
                }
                if (width < 1) width = 1;
                if (height < 1) height = 1;

                // ウィンドウを配置
                if (IsWindow((IntPtr)window.hWnd) != 0)
                {
                    SetWindowPos(window.hWnd, HWND_NOTOPMOST, left, top, width, height, SWP_SHOWWINDOW);

                    relocationLog.AppendFormat(
                        "\"{0}\",\"{1}\", ({2}, {3}, {4}, {5}) -> ({6}, {7}, {8}, {9})\r\n",
                        Regex.Escape(window.windowText ?? ""),
                        Regex.Escape(window.className ?? ""),
                        oldLeft,
                        oldTop,
                        oldWidth,
                        oldHeight,
                        left,
                        top,
                        width,
                        height);
                }
            }

            info.Text = relocationLog.Length > 0
                ? relocationLog.ToString()
                : "対象ウィンドウがありません。";
        }

        private void btnLocationSet_Click(object sender, EventArgs e)
        {
            location_set(System.Windows.Forms.Screen.AllScreens.Length);
        }

        private void btnOneDisplaySet_Click(object sender, EventArgs e)
        {
            location_set(1);
        }


        private static string windows_list = "";

        private static bool EnumWindowCallBack2(IntPtr hWnd, IntPtr lparam)
        {
            //ウィンドウのタイトルの長さを取得する
            int textLen = GetWindowTextLength(hWnd);
            if (0 < textLen && IsWindowVisible(hWnd))
            {
                //ウィンドウのタイトルを取得する
                StringBuilder tsb = new StringBuilder(textLen + 1);
                GetWindowText(hWnd, tsb, tsb.Capacity);

                //ウィンドウのクラス名を取得する
                StringBuilder csb = new StringBuilder(256);
                GetClassName(hWnd, csb, csb.Capacity);

                wRECT rect;
                GetWindowRect(hWnd, out rect);
                windows_list += string.Format("\"{1}\",\"{2}\", {3}, {4}, {5}, {6}\r\n", "", Regex.Escape(tsb.ToString()), Regex.Escape(csb.ToString()), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top);
            }

            //すべてのウィンドウを列挙する
            return true;
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct wPOINT
        {
            public int X;
            public int Y;
        }
        [StructLayout(LayoutKind.Sequential)]
        public struct wSIZE
        {
            public int Width;
            public int Height;
        }
        [StructLayout(LayoutKind.Sequential)]
        private struct wRECT
        {
            public int left;
            public int top;
            public int right;
            public int bottom;
        }

        List<string[]> setpos_list = new List<string[]>();

        /// <summary>
        /// 指定された文字列をウィンドウのタイトルとクラス名に含んでいるウィンドウハンドルをすべて取得する。
        /// </summary>
        /// <param name="windowText">ウィンドウのタイトルに含むべき文字列。
        /// nullを指定すると、classNameだけで検索する。</param>
        /// <param name="className">ウィンドウが属するクラス名に含むべき文字列。
        /// nullを指定すると、windowTextだけで検索する。</param>
        private delegate bool EnumWindowsDelegate(IntPtr hWnd, IntPtr lparam);

        [DllImport("user32.dll")]
        [return: MarshalAs(UnmanagedType.Bool)]
        private extern static bool EnumWindows(EnumWindowsDelegate lpEnumFunc, IntPtr lparam);

        [DllImport("user32.dll")]
        private static extern bool GetWindowRect(IntPtr hwnd, out wRECT lpRect);

        [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
        private static extern int GetWindowText(IntPtr hWnd,
            StringBuilder lpString, int nMaxCount);

        [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
        private static extern int GetWindowTextLength(IntPtr hWnd);

        [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
        private static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);

        [DllImport("user32.dll", SetLastError = true)]
        private static extern int GetWindowThreadProcessId(IntPtr hWnd, out int lpdwProcessId);

        [DllImport("user32.dll", CharSet = CharSet.Auto)]
        static extern int IsWindow(IntPtr hWnd);

        [DllImport("user32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool SetWindowPos(IntPtr hWnd, int hWndInsertAfter, int x, int y, int cx, int cy, int uFlags);

        [DllImport("user32.dll")]
        private static extern bool IsWindowVisible(IntPtr hWnd);

        [DllImport("user32.dll")]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool GetWindowPlacement(IntPtr hWnd, ref WINDOWPLACEMENT lpwndpl);

        private struct WINDOWPLACEMENT
        {
            public int length;
            public int flags;
            public int showCmd;
            public System.Drawing.Point ptMinPosition;
            public System.Drawing.Point ptMaxPosition;
            public System.Drawing.Rectangle rcNormalPosition;
        }

        private const int SWP_NOSIZE = 0x0001;
        private const int SWP_NOMOVE = 0x0002;
        private const int SWP_SHOWWINDOW = 0x0040;
        private const int HWND_TOPMOST = -1;
        private const int HWND_NOTOPMOST = -2;

        private void button2_Click(object sender, EventArgs e)
        {
            SetThreadExecutionState(ExecutionState.Continuous);
            preventSleep.Checked = false;
            timer1.Enabled = preventSleep.Checked;
            SendMessage(HWND_BROADCAST, WM_SYSCOMMAND, (IntPtr)SC_MONITORPOWER, (IntPtr)MonitorShutoff);
        }
    }
}

