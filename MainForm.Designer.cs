namespace SleepFrame
{
    partial class MainForm
    {
        /// <summary>
        /// Required designer variable.
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary>
        /// Clean up any resources being used.
        /// </summary>
        /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Windows Form Designer generated code

        /// <summary>
        /// Required method for Designer support - do not modify
        /// the contents of this method with the code editor.
        /// </summary>
        private void InitializeComponent()
        {
            this.components = new System.ComponentModel.Container();
            System.ComponentModel.ComponentResourceManager resources = new System.ComponentModel.ComponentResourceManager(typeof(MainForm));
            this._btnStartOrStop = new System.Windows.Forms.Button();
            this.label1 = new System.Windows.Forms.Label();
            this._chMacros = new System.Windows.Forms.ComboBox();
            this.groupBox1 = new System.Windows.Forms.GroupBox();
            this._lbMacroDescription = new System.Windows.Forms.Label();
            this.gbStart_HotKey = new System.Windows.Forms.GroupBox();
            this.cbKeyToo = new System.Windows.Forms.ComboBox();
            this.cbKeyOn = new System.Windows.Forms.ComboBox();
            this.label4 = new System.Windows.Forms.Label();
            this._niApp = new System.Windows.Forms.NotifyIcon(this.components);
            this._lblTimer = new System.Windows.Forms.Label();
            this.gbStart_HotKey.SuspendLayout();
            this.SuspendLayout();
            // 
            // _btnStartOrStop
            // 
            this._btnStartOrStop.Location = new System.Drawing.Point(12, 52);
            this._btnStartOrStop.Name = "_btnStartOrStop";
            this._btnStartOrStop.Size = new System.Drawing.Size(164, 23);
            this._btnStartOrStop.TabIndex = 1;
            this._btnStartOrStop.Text = "Start";
            this._btnStartOrStop.UseVisualStyleBackColor = true;
            this._btnStartOrStop.Click += new System.EventHandler(this._btnStartOrStop_Click);
            // 
            // label1
            // 
            this.label1.AutoSize = true;
            this.label1.Location = new System.Drawing.Point(9, 8);
            this.label1.Name = "label1";
            this.label1.Size = new System.Drawing.Size(44, 13);
            this.label1.TabIndex = 2;
            this.label1.Text = "Macro\'s";
            // 
            // _chMacros
            // 
            this._chMacros.DropDownStyle = System.Windows.Forms.ComboBoxStyle.DropDownList;
            this._chMacros.FormattingEnabled = true;
            this._chMacros.Location = new System.Drawing.Point(12, 25);
            this._chMacros.Name = "_chMacros";
            this._chMacros.Size = new System.Drawing.Size(164, 21);
            this._chMacros.TabIndex = 3;
            this._chMacros.SelectedIndexChanged += new System.EventHandler(this._chMacros_SelectedIndexChanged);
            // 
            // groupBox1
            // 
            this.groupBox1.Location = new System.Drawing.Point(12, 99);
            this.groupBox1.Name = "groupBox1";
            this.groupBox1.Size = new System.Drawing.Size(670, 210);
            this.groupBox1.TabIndex = 4;
            this.groupBox1.TabStop = false;
            this.groupBox1.Text = "Macro Settings";
            // 
            // _lbMacroDescription
            // 
            this._lbMacroDescription.Location = new System.Drawing.Point(338, 8);
            this._lbMacroDescription.Name = "_lbMacroDescription";
            this._lbMacroDescription.Size = new System.Drawing.Size(244, 67);
            this._lbMacroDescription.TabIndex = 5;
            // 
            // gbStart_HotKey
            // 
            this.gbStart_HotKey.Controls.Add(this.cbKeyToo);
            this.gbStart_HotKey.Controls.Add(this.cbKeyOn);
            this.gbStart_HotKey.Controls.Add(this.label4);
            this.gbStart_HotKey.Location = new System.Drawing.Point(192, 8);
            this.gbStart_HotKey.Name = "gbStart_HotKey";
            this.gbStart_HotKey.Size = new System.Drawing.Size(127, 68);
            this.gbStart_HotKey.TabIndex = 6;
            this.gbStart_HotKey.TabStop = false;
            this.gbStart_HotKey.Text = "HotKey";
            // 
            // cbKeyToo
            // 
            this.cbKeyToo.DropDownStyle = System.Windows.Forms.ComboBoxStyle.DropDownList;
            this.cbKeyToo.FormattingEnabled = true;
            this.cbKeyToo.Items.AddRange(new object[] {
            "F1",
            "F2",
            "F3",
            "F4",
            "F5",
            "F6",
            "F7",
            "F8",
            "F9",
            "F10",
            "F11",
            "F12"});
            this.cbKeyToo.Location = new System.Drawing.Point(18, 39);
            this.cbKeyToo.Name = "cbKeyToo";
            this.cbKeyToo.Size = new System.Drawing.Size(103, 21);
            this.cbKeyToo.TabIndex = 1;
            // 
            // cbKeyOn
            // 
            this.cbKeyOn.DropDownStyle = System.Windows.Forms.ComboBoxStyle.DropDownList;
            this.cbKeyOn.FormattingEnabled = true;
            this.cbKeyOn.Items.AddRange(new object[] {
            "None",
            "Ctrl"});
            this.cbKeyOn.Location = new System.Drawing.Point(18, 15);
            this.cbKeyOn.Name = "cbKeyOn";
            this.cbKeyOn.Size = new System.Drawing.Size(103, 21);
            this.cbKeyOn.TabIndex = 1;
            // 
            // label4
            // 
            this.label4.AutoSize = true;
            this.label4.Location = new System.Drawing.Point(3, 31);
            this.label4.Name = "label4";
            this.label4.Size = new System.Drawing.Size(13, 13);
            this.label4.TabIndex = 0;
            this.label4.Text = "+";
            // 
            // _niApp
            // 
            this._niApp.BalloonTipText = "SleepFrame";
            this._niApp.BalloonTipTitle = "SleepFrame";
            this._niApp.Icon = ((System.Drawing.Icon)(resources.GetObject("_niApp.Icon")));
            this._niApp.Text = "SleepFrame";
            this._niApp.Visible = true;
            // 
            // _lblTimer
            // 
            this._lblTimer.AutoSize = true;
            this._lblTimer.Location = new System.Drawing.Point(13, 80);
            this._lblTimer.Name = "_lblTimer";
            this._lblTimer.Size = new System.Drawing.Size(35, 13);
            this._lblTimer.TabIndex = 7;
            this._lblTimer.Text = "Idile...";
            // 
            // MainForm
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(6F, 13F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.ClientSize = new System.Drawing.Size(697, 325);
            this.Controls.Add(this._lblTimer);
            this.Controls.Add(this.gbStart_HotKey);
            this.Controls.Add(this._lbMacroDescription);
            this.Controls.Add(this.groupBox1);
            this.Controls.Add(this._chMacros);
            this.Controls.Add(this.label1);
            this.Controls.Add(this._btnStartOrStop);
            this.Icon = ((System.Drawing.Icon)(resources.GetObject("$this.Icon")));
            this.Name = "MainForm";
            this.Text = "SleepFrame";
            this.gbStart_HotKey.ResumeLayout(false);
            this.gbStart_HotKey.PerformLayout();
            this.ResumeLayout(false);
            this.PerformLayout();

        }

        #endregion
        private System.Windows.Forms.Button _btnStartOrStop;
        private System.Windows.Forms.Label label1;
        private System.Windows.Forms.ComboBox _chMacros;
        private System.Windows.Forms.GroupBox groupBox1;
        private System.Windows.Forms.Label _lbMacroDescription;
        private System.Windows.Forms.GroupBox gbStart_HotKey;
        private System.Windows.Forms.ComboBox cbKeyToo;
        private System.Windows.Forms.ComboBox cbKeyOn;
        private System.Windows.Forms.Label label4;
        private System.Windows.Forms.NotifyIcon _niApp;
        private System.Windows.Forms.Label _lblTimer;
    }
}

