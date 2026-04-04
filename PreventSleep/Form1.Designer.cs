namespace PreventSleep
{
    partial class Form1
    {
        /// <summary>
        /// 必要なデザイナー変数です。
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary>
        /// 使用中のリソースをすべてクリーンアップします。
        /// </summary>
        /// <param name="disposing">マネージド リソースを破棄する場合は true を指定し、その他の場合は false を指定します。</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Windows フォーム デザイナーで生成されたコード

        /// <summary>
        /// デザイナー サポートに必要なメソッドです。このメソッドの内容を
        /// コード エディターで変更しないでください。
        /// </summary>
        private void InitializeComponent()
        {
            this.components = new System.ComponentModel.Container();
            this.timer1 = new System.Windows.Forms.Timer(this.components);
            this.timer2 = new System.Windows.Forms.Timer(this.components);
            this.preventSleep = new System.Windows.Forms.CheckBox();
            this.info = new System.Windows.Forms.TextBox();
            this.btnLocationSet = new System.Windows.Forms.Button();
            this.btnListWindows = new System.Windows.Forms.Button();
            this.menuStrip1 = new System.Windows.Forms.MenuStrip();
            this.button1 = new System.Windows.Forms.Button();
            this.button2 = new System.Windows.Forms.Button();
            this.SuspendLayout();
            // 
            // timer1
            // 
            this.timer1.Interval = 30000;
            this.timer1.Tick += new System.EventHandler(this.timer1_Tick);
            // 
            // timer2
            // 
            this.timer2.Tick += new System.EventHandler(this.timer2_Tick);
            // 
            // preventSleep
            // 
            this.preventSleep.AutoSize = true;
            this.preventSleep.Checked = true;
            this.preventSleep.CheckState = System.Windows.Forms.CheckState.Checked;
            this.preventSleep.Location = new System.Drawing.Point(5, 6);
            this.preventSleep.Name = "preventSleep";
            this.preventSleep.Size = new System.Drawing.Size(95, 16);
            this.preventSleep.TabIndex = 0;
            this.preventSleep.Text = "Prevent Sleep";
            this.preventSleep.UseVisualStyleBackColor = true;
            this.preventSleep.CheckedChanged += new System.EventHandler(this.preventSleep_CheckedChanged);
            // 
            // info
            // 
            this.info.Location = new System.Drawing.Point(119, 6);
            this.info.Multiline = true;
            this.info.Name = "info";
            this.info.ReadOnly = true;
            this.info.ScrollBars = System.Windows.Forms.ScrollBars.Both;
            this.info.Size = new System.Drawing.Size(269, 85);
            this.info.TabIndex = 2;
            this.info.WordWrap = false;
            // 
            // btnLocationSet
            // 
            this.btnLocationSet.Location = new System.Drawing.Point(5, 56);
            this.btnLocationSet.Name = "btnLocationSet";
            this.btnLocationSet.Size = new System.Drawing.Size(72, 35);
            this.btnLocationSet.TabIndex = 3;
            this.btnLocationSet.Text = "座標セット";
            this.btnLocationSet.UseVisualStyleBackColor = true;
            this.btnLocationSet.Click += new System.EventHandler(this.btnLocationSet_Click);
            // 
            // btnListWindows
            // 
            this.btnListWindows.Font = new System.Drawing.Font("MS UI Gothic", 7F, System.Drawing.FontStyle.Regular, System.Drawing.GraphicsUnit.Point, ((byte)(128)));
            this.btnListWindows.Location = new System.Drawing.Point(5, 29);
            this.btnListWindows.Name = "btnListWindows";
            this.btnListWindows.Size = new System.Drawing.Size(72, 22);
            this.btnListWindows.TabIndex = 4;
            this.btnListWindows.Text = "ウィンドウ列挙";
            this.btnListWindows.UseVisualStyleBackColor = true;
            this.btnListWindows.Click += new System.EventHandler(this.btnListWindows_Click);
            // 
            // menuStrip1
            // 
            this.menuStrip1.ImageScalingSize = new System.Drawing.Size(24, 24);
            this.menuStrip1.Location = new System.Drawing.Point(0, 0);
            this.menuStrip1.Name = "menuStrip1";
            this.menuStrip1.Padding = new System.Windows.Forms.Padding(4, 1, 0, 1);
            this.menuStrip1.Size = new System.Drawing.Size(394, 24);
            this.menuStrip1.TabIndex = 5;
            this.menuStrip1.Text = "menuStrip1";
            // 
            // button1
            // 
            this.button1.Location = new System.Drawing.Point(82, 56);
            this.button1.Margin = new System.Windows.Forms.Padding(2);
            this.button1.Name = "button1";
            this.button1.Size = new System.Drawing.Size(18, 35);
            this.button1.TabIndex = 6;
            this.button1.Text = "1";
            this.button1.UseVisualStyleBackColor = true;
            this.button1.Click += new System.EventHandler(this.btnOneDisplaySet_Click);
            // 
            // button2
            // 
            this.button2.Location = new System.Drawing.Point(82, 23);
            this.button2.Margin = new System.Windows.Forms.Padding(2);
            this.button2.Name = "button2";
            this.button2.Size = new System.Drawing.Size(18, 35);
            this.button2.TabIndex = 7;
            this.button2.Text = "X";
            this.button2.UseVisualStyleBackColor = true;
            this.button2.Click += new System.EventHandler(this.button2_Click);
            // 
            // Form1
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(6F, 12F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.ClientSize = new System.Drawing.Size(394, 99);
            this.Controls.Add(this.button2);
            this.Controls.Add(this.button1);
            this.Controls.Add(this.btnListWindows);
            this.Controls.Add(this.btnLocationSet);
            this.Controls.Add(this.info);
            this.Controls.Add(this.preventSleep);
            this.Controls.Add(this.menuStrip1);
            this.MainMenuStrip = this.menuStrip1;
            this.Icon = new System.Drawing.Icon(System.IO.Path.Combine(System.AppDomain.CurrentDomain.BaseDirectory, "app.ico"));
            this.Name = "Form1";
            this.Text = "PreventSleep";
            this.TopMost = true;
            this.Load += new System.EventHandler(this.Form1_Load);
            this.ResumeLayout(false);
            this.PerformLayout();

        }

        #endregion

        private System.Windows.Forms.Timer timer1;
        private System.Windows.Forms.Timer timer2;
        private System.Windows.Forms.CheckBox preventSleep;
        private System.Windows.Forms.TextBox info;
        private System.Windows.Forms.Button btnLocationSet;
        private System.Windows.Forms.Button btnListWindows;
        private System.Windows.Forms.MenuStrip menuStrip1;
        private System.Windows.Forms.Button button1;
        private System.Windows.Forms.Button button2;
    }
}

